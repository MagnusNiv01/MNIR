#![forbid(unsafe_code)]

//! Canonical `.mnir` format 1.0 encoding and decoding.
//!
//! `minicbor` is deliberately confined to this crate and used only as a
//! low-level CBOR reader/writer. The MNIR wire schema is implemented manually.

use std::cmp::Ordering;
use std::error::Error;
use std::fmt;

use minicbor::{Decoder, Encoder};
use mnir_core::{
    AllocationCounterState, AllocationNamespaceId, BlockId, ExpressionId, ExpressionKind,
    FunctionId, IntrinsicType, MnirProgram, ParameterId, PersistenceBlock, PersistenceBody,
    PersistenceDomainType, PersistenceEntityId, PersistenceExpression, PersistenceExpressionKind,
    PersistenceFunction, PersistenceModule, PersistenceParameter, PersistencePresentation,
    PersistenceProgram, PersistenceSnapshot, PersistenceTerminator, PersistenceValueType,
    ProgramId, RevisionCounterState, RevisionId, Terminator, TypeId, ValidatedPersistenceState,
    ValueType,
};

const MAGIC: [u8; 8] = [0x89, b'M', b'N', b'I', b'R', 0x0d, 0x0a, 0x1a];
const FORMAT_MAJOR: u16 = 1;
const FORMAT_MINOR: u16 = 0;
const ENVELOPE_LEN: usize = 12;
const MAX_FILE_SIZE: usize = 1 << 30;
const MAX_ITEM_SIZE: u64 = 1 << 28;
const MAX_COLLECTION_COUNT: u64 = 1 << 24;
const MAX_DEPTH: u8 = 16;

/// Canonical encoding failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    InvalidCheckpoint,
    ResourceLimitExceeded,
}

impl fmt::Display for EncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidCheckpoint => "invalid persistence checkpoint",
            Self::ResourceLimitExceeded => "persistence resource limit exceeded",
        })
    }
}

impl Error for EncodeError {}

/// Machine-readable canonical decoding failure categories (`MNIR-SER-094`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeError {
    UnsupportedFormatVersion,
    MalformedEncoding,
    NonCanonicalEncoding,
    ResourceLimitExceeded,
    InvalidWireSchema,
    StructurallyInvalidProgram,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedFormatVersion => "unsupported MNIR format version",
            Self::MalformedEncoding => "malformed MNIR encoding",
            Self::NonCanonicalEncoding => "non-canonical MNIR encoding",
            Self::ResourceLimitExceeded => "MNIR persistence resource limit exceeded",
            Self::InvalidWireSchema => "invalid MNIR wire schema",
            Self::StructurallyInvalidProgram => "structurally invalid MNIR Program",
        })
    }
}

impl Error for DecodeError {}

/// Fully decoded and structurally validated immutable persistence state.
///
/// Decoding does not grant mutable allocation authority. [`Self::activate`]
/// atomically claims the unique process-local authority lease.
#[derive(Debug)]
pub struct DecodedProgram {
    validated: ValidatedPersistenceState,
}

impl DecodedProgram {
    #[must_use]
    pub const fn program_id(&self) -> ProgramId {
        self.validated.program_id()
    }

    #[must_use]
    pub fn revision_id(&self) -> RevisionId {
        self.validated.revision_id()
    }

    #[must_use]
    pub const fn revision_counter_state(&self) -> RevisionCounterState {
        self.validated.revision_counter_state()
    }

    #[must_use]
    pub const fn allocation_namespace_id(&self) -> AllocationNamespaceId {
        self.validated.allocation_namespace_id()
    }

    #[must_use]
    pub const fn allocation_counter_state(&self) -> AllocationCounterState {
        self.validated.allocation_counter_state()
    }

    #[must_use]
    pub fn module_count(&self) -> usize {
        self.validated.modules().count()
    }

    /// Returns an immutable checkpoint suitable for canonical re-encoding.
    #[must_use]
    pub fn persistence_snapshot(&self) -> PersistenceSnapshot {
        self.validated.persistence_snapshot()
    }

    /// Claims this checkpoint as the unique mutable continuation in this
    /// process. Repeated, concurrent, or stale activation fails atomically.
    pub fn activate(self) -> Result<MnirProgram, DecodeError> {
        MnirProgram::activate_persistence(self.validated)
            .map_err(|_| DecodeError::StructurallyInvalidProgram)
    }
}

/// Encodes one immutable current-lineage checkpoint into exact format-1.0 bytes.
pub fn encode(snapshot: &PersistenceSnapshot) -> Result<Vec<u8>, EncodeError> {
    let mut encoder = Encoder::new(LimitedVec::default());
    encode_payload(&mut encoder, snapshot)?;
    let payload = encoder.into_writer().bytes;
    let total_len = ENVELOPE_LEN
        .checked_add(payload.len())
        .ok_or(EncodeError::ResourceLimitExceeded)?;
    check_file_size_for_encode(total_len)?;
    let mut output = Vec::with_capacity(total_len);
    output.extend_from_slice(&MAGIC);
    output.extend_from_slice(&FORMAT_MAJOR.to_be_bytes());
    output.extend_from_slice(&FORMAT_MINOR.to_be_bytes());
    output.extend_from_slice(&payload);
    Ok(output)
}

/// Decodes and structurally validates one exact canonical format-1.0 file into
/// an immutable observation. Mutable continuation requires explicit activation.
pub fn decode(bytes: &[u8]) -> Result<DecodedProgram, DecodeError> {
    validate_envelope(bytes)?;
    let payload = &bytes[ENVELOPE_LEN..];
    validate_canonical_cbor(payload)?;
    let candidate = decode_payload(payload)?;
    let validated = MnirProgram::validate_persistence(candidate)
        .map_err(|_| DecodeError::StructurallyInvalidProgram)?;
    Ok(DecodedProgram { validated })
}

#[derive(Debug, Default)]
struct LimitedVec {
    bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
struct OutputLimitExceeded;

impl minicbor::encode::Write for LimitedVec {
    type Error = OutputLimitExceeded;

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        let limit = MAX_FILE_SIZE - ENVELOPE_LEN;
        let Some(new_len) = self.bytes.len().checked_add(bytes.len()) else {
            return Err(OutputLimitExceeded);
        };
        if new_len > limit {
            return Err(OutputLimitExceeded);
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
}

type Enc = Encoder<LimitedVec>;

macro_rules! emit {
    ($expression:expr) => {
        $expression.map_err(|error| {
            if error.is_write() {
                EncodeError::ResourceLimitExceeded
            } else {
                EncodeError::InvalidCheckpoint
            }
        })?
    };
}

fn encode_payload(encoder: &mut Enc, snapshot: &PersistenceSnapshot) -> Result<(), EncodeError> {
    emit!(encoder.array(5));
    encode_program_id(encoder, snapshot.program_id())?;
    emit!(encoder.u64(snapshot.revision_id().persistent_value()));
    encode_revision_counter(encoder, snapshot.revision_counter_state())?;
    encode_allocation_authority(
        encoder,
        snapshot.allocation_namespace_id(),
        snapshot.allocation_counter_state(),
    )?;

    let mut modules: Vec<_> = snapshot.modules().collect();
    modules
        .sort_by_key(|module| entity_sort_key(module.id().namespace_id(), module.id().counter()));
    encode_count(encoder, modules.len())?;
    for module in modules {
        emit!(encoder.array(4));
        encode_module_id(encoder, module.id())?;
        encode_presentation(encoder, module.presentation())?;

        let mut domain_types: Vec<_> = module.domain_types().collect();
        domain_types.sort_by_key(|domain| {
            entity_sort_key(domain.id().namespace_id(), domain.id().counter())
        });
        encode_count(encoder, domain_types.len())?;
        for domain_type in domain_types {
            emit!(encoder.array(3));
            encode_entity_id(
                encoder,
                domain_type.id().namespace_id(),
                domain_type.id().counter(),
            )?;
            emit!(encoder.u8(intrinsic_tag(domain_type.representation())));
            encode_presentation(encoder, domain_type.presentation())?;
        }

        let mut functions: Vec<_> = module.functions().collect();
        functions.sort_by_key(|function| {
            entity_sort_key(function.id().namespace_id(), function.id().counter())
        });
        encode_count(encoder, functions.len())?;
        for function in functions {
            emit!(encoder.array(5));
            encode_entity_id(
                encoder,
                function.id().namespace_id(),
                function.id().counter(),
            )?;
            encode_value_type(encoder, *function.return_type())?;
            encode_count(encoder, function.parameters().len())?;
            for parameter in function.parameters() {
                emit!(encoder.array(3));
                encode_entity_id(
                    encoder,
                    parameter.id().namespace_id(),
                    parameter.id().counter(),
                )?;
                encode_value_type(encoder, *parameter.value_type())?;
                encode_presentation(encoder, parameter.presentation())?;
            }
            encode_body(encoder, function.body())?;
            encode_presentation(encoder, function.presentation())?;
        }
    }
    Ok(())
}

fn encode_body(
    encoder: &mut Enc,
    body: Option<&mnir_core::FunctionBody>,
) -> Result<(), EncodeError> {
    let Some(body) = body else {
        emit!(encoder.array(1));
        emit!(encoder.u8(0));
        return Ok(());
    };
    emit!(encoder.array(2));
    emit!(encoder.u8(1));
    emit!(encoder.array(2));
    encode_entity_id(
        encoder,
        body.entry_block_id().namespace_id(),
        body.entry_block_id().counter(),
    )?;
    let mut blocks: Vec<_> = body.blocks().collect();
    blocks.sort_by_key(|block| entity_sort_key(block.id().namespace_id(), block.id().counter()));
    encode_count(encoder, blocks.len())?;
    for block in blocks {
        emit!(encoder.array(4));
        encode_entity_id(encoder, block.id().namespace_id(), block.id().counter())?;
        let mut expressions: Vec<_> = block.expressions().collect();
        expressions.sort_by_key(|expression| {
            entity_sort_key(expression.id().namespace_id(), expression.id().counter())
        });
        encode_count(encoder, expressions.len())?;
        for expression in expressions {
            emit!(encoder.array(2));
            encode_entity_id(
                encoder,
                expression.id().namespace_id(),
                expression.id().counter(),
            )?;
            encode_expression_kind(encoder, expression.kind())?;
        }
        encode_count(encoder, block.effect_sequence().len())?;
        for expression_id in block.effect_sequence() {
            encode_expression_id(encoder, *expression_id)?;
        }
        let terminator = block.terminator().ok_or(EncodeError::InvalidCheckpoint)?;
        encode_terminator(encoder, *terminator)?;
    }
    Ok(())
}

fn encode_expression_kind(encoder: &mut Enc, kind: &ExpressionKind) -> Result<(), EncodeError> {
    match kind {
        ExpressionKind::Int32Literal(value) => {
            emit!(encoder.array(2));
            emit!(encoder.u8(0));
            emit!(encoder.i32(*value));
            return Ok(());
        }
        ExpressionKind::Int64Literal(value) => {
            emit!(encoder.array(2));
            emit!(encoder.u8(1));
            emit!(encoder.i64(*value));
            return Ok(());
        }
        ExpressionKind::BoolLiteral(value) => {
            emit!(encoder.array(2));
            emit!(encoder.u8(2));
            emit!(encoder.bool(*value));
            return Ok(());
        }
        ExpressionKind::UnitLiteral => {
            emit!(encoder.array(1));
            emit!(encoder.u8(3));
            return Ok(());
        }
        ExpressionKind::TextLiteral(value) => {
            check_item_len(value.len())?;
            emit!(encoder.array(2));
            emit!(encoder.u8(4));
            emit!(encoder.str(value));
            return Ok(());
        }
        ExpressionKind::BytesLiteral(value) => {
            check_item_len(value.len())?;
            emit!(encoder.array(2));
            emit!(encoder.u8(5));
            emit!(encoder.bytes(value));
            return Ok(());
        }
        ExpressionKind::ParameterReference(id) => {
            emit!(encoder.array(2));
            emit!(encoder.u8(6));
            encode_parameter_id(encoder, *id)?;
            return Ok(());
        }
        ExpressionKind::DomainConstruct { type_id, value } => {
            emit!(encoder.array(3));
            emit!(encoder.u8(19));
            encode_type_id(encoder, *type_id)?;
            encode_expression_id(encoder, *value)?;
            return Ok(());
        }
        ExpressionKind::DomainProject { value } => {
            emit!(encoder.array(2));
            emit!(encoder.u8(20));
            encode_expression_id(encoder, *value)?;
            return Ok(());
        }
        ExpressionKind::Call { target, arguments } => {
            emit!(encoder.array(3));
            emit!(encoder.u8(18));
            encode_function_id(encoder, *target)?;
            encode_count(encoder, arguments.len())?;
            for argument in arguments {
                encode_expression_id(encoder, *argument)?;
            }
            return Ok(());
        }
        ExpressionKind::Add { left, right } => encode_binary_header(encoder, 7, *left, *right)?,
        ExpressionKind::Subtract { left, right } => {
            encode_binary_header(encoder, 8, *left, *right)?
        }
        ExpressionKind::Multiply { left, right } => {
            encode_binary_header(encoder, 9, *left, *right)?
        }
        ExpressionKind::Divide { left, right } => encode_binary_header(encoder, 10, *left, *right)?,
        ExpressionKind::Remainder { left, right } => {
            encode_binary_header(encoder, 11, *left, *right)?
        }
        ExpressionKind::Equal { left, right } => encode_binary_header(encoder, 12, *left, *right)?,
        ExpressionKind::NotEqual { left, right } => {
            encode_binary_header(encoder, 13, *left, *right)?
        }
        ExpressionKind::LessThan { left, right } => {
            encode_binary_header(encoder, 14, *left, *right)?
        }
        ExpressionKind::LessThanOrEqual { left, right } => {
            encode_binary_header(encoder, 15, *left, *right)?
        }
        ExpressionKind::GreaterThan { left, right } => {
            encode_binary_header(encoder, 16, *left, *right)?
        }
        ExpressionKind::GreaterThanOrEqual { left, right } => {
            encode_binary_header(encoder, 17, *left, *right)?
        }
    }
    Ok(())
}

fn encode_binary_header(
    encoder: &mut Enc,
    tag: u8,
    left: ExpressionId,
    right: ExpressionId,
) -> Result<(), EncodeError> {
    emit!(encoder.array(3));
    emit!(encoder.u8(tag));
    encode_expression_id(encoder, left)?;
    encode_expression_id(encoder, right)
}

fn encode_terminator(encoder: &mut Enc, terminator: Terminator) -> Result<(), EncodeError> {
    match terminator {
        Terminator::Return { expression } => {
            emit!(encoder.array(2));
            emit!(encoder.u8(0));
            encode_expression_id(encoder, expression)?;
        }
        Terminator::Branch {
            condition,
            true_block,
            false_block,
        } => {
            emit!(encoder.array(4));
            emit!(encoder.u8(1));
            encode_expression_id(encoder, condition)?;
            encode_block_id(encoder, true_block)?;
            encode_block_id(encoder, false_block)?;
        }
    }
    Ok(())
}

fn encode_presentation(
    encoder: &mut Enc,
    presentation: &mnir_core::PresentationMetadata,
) -> Result<(), EncodeError> {
    emit!(encoder.array(2));
    encode_string_option(encoder, presentation.preferred_name())?;
    encode_string_option(encoder, presentation.documentation())
}

fn encode_string_option(encoder: &mut Enc, value: Option<&str>) -> Result<(), EncodeError> {
    match value {
        None => {
            emit!(encoder.array(1));
            emit!(encoder.u8(0));
        }
        Some(value) => {
            check_item_len(value.len())?;
            emit!(encoder.array(2));
            emit!(encoder.u8(1));
            emit!(encoder.str(value));
        }
    }
    Ok(())
}

fn encode_value_type(encoder: &mut Enc, value_type: ValueType) -> Result<(), EncodeError> {
    emit!(encoder.array(2));
    match value_type {
        ValueType::Intrinsic(intrinsic) => {
            emit!(encoder.u8(0));
            emit!(encoder.u8(intrinsic_tag(intrinsic)));
        }
        ValueType::Domain(type_id) => {
            emit!(encoder.u8(1));
            encode_type_id(encoder, type_id)?;
        }
    }
    Ok(())
}

const fn intrinsic_tag(intrinsic: IntrinsicType) -> u8 {
    match intrinsic {
        IntrinsicType::Int32 => 0,
        IntrinsicType::Int64 => 1,
        IntrinsicType::Bool => 2,
        IntrinsicType::Unit => 3,
        IntrinsicType::Text => 4,
        IntrinsicType::Bytes => 5,
    }
}

fn encode_revision_counter(
    encoder: &mut Enc,
    state: RevisionCounterState,
) -> Result<(), EncodeError> {
    match state {
        RevisionCounterState::Available(next) if next != 0 => {
            emit!(encoder.array(2));
            emit!(encoder.u8(0));
            emit!(encoder.u64(next));
        }
        RevisionCounterState::Exhausted => {
            emit!(encoder.array(1));
            emit!(encoder.u8(1));
        }
        RevisionCounterState::Available(_) => return Err(EncodeError::InvalidCheckpoint),
    }
    Ok(())
}

fn encode_allocation_authority(
    encoder: &mut Enc,
    namespace: AllocationNamespaceId,
    state: AllocationCounterState,
) -> Result<(), EncodeError> {
    encode_raw_allocation_authority(encoder, namespace.persistent_bytes(), state)
}

fn encode_raw_allocation_authority(
    encoder: &mut Enc,
    namespace: [u8; 16],
    state: AllocationCounterState,
) -> Result<(), EncodeError> {
    emit!(encoder.array(2));
    emit!(encoder.bytes(&namespace));
    match state {
        AllocationCounterState::Available(next) => {
            emit!(encoder.array(2));
            emit!(encoder.u8(0));
            emit!(encoder.u64(next));
        }
        AllocationCounterState::Exhausted => {
            emit!(encoder.array(1));
            emit!(encoder.u8(1));
        }
    }
    Ok(())
}

fn encode_program_id(encoder: &mut Enc, id: ProgramId) -> Result<(), EncodeError> {
    emit!(encoder.bytes(&id.persistent_bytes()));
    Ok(())
}

fn encode_entity_id(
    encoder: &mut Enc,
    namespace: AllocationNamespaceId,
    counter: u64,
) -> Result<(), EncodeError> {
    encode_raw_entity_id(encoder, namespace.persistent_bytes(), counter)
}

fn encode_raw_entity_id(
    encoder: &mut Enc,
    namespace: [u8; 16],
    counter: u64,
) -> Result<(), EncodeError> {
    emit!(encoder.array(2));
    emit!(encoder.bytes(&namespace));
    emit!(encoder.u64(counter));
    Ok(())
}

fn encode_type_id(encoder: &mut Enc, id: TypeId) -> Result<(), EncodeError> {
    encode_entity_id(encoder, id.namespace_id(), id.counter())
}
fn encode_module_id(encoder: &mut Enc, id: mnir_core::ModuleId) -> Result<(), EncodeError> {
    encode_entity_id(encoder, id.namespace_id(), id.counter())
}
fn encode_function_id(encoder: &mut Enc, id: FunctionId) -> Result<(), EncodeError> {
    encode_entity_id(encoder, id.namespace_id(), id.counter())
}
fn encode_parameter_id(encoder: &mut Enc, id: ParameterId) -> Result<(), EncodeError> {
    encode_entity_id(encoder, id.namespace_id(), id.counter())
}
fn encode_block_id(encoder: &mut Enc, id: BlockId) -> Result<(), EncodeError> {
    encode_entity_id(encoder, id.namespace_id(), id.counter())
}
fn encode_expression_id(encoder: &mut Enc, id: ExpressionId) -> Result<(), EncodeError> {
    encode_entity_id(encoder, id.namespace_id(), id.counter())
}

fn encode_count(encoder: &mut Enc, len: usize) -> Result<(), EncodeError> {
    if len as u64 > MAX_COLLECTION_COUNT {
        return Err(EncodeError::ResourceLimitExceeded);
    }
    emit!(encoder.array(len as u64));
    Ok(())
}

fn check_item_len(len: usize) -> Result<(), EncodeError> {
    if len as u64 > MAX_ITEM_SIZE {
        Err(EncodeError::ResourceLimitExceeded)
    } else {
        Ok(())
    }
}

fn check_file_size_for_encode(len: usize) -> Result<(), EncodeError> {
    if len > MAX_FILE_SIZE {
        Err(EncodeError::ResourceLimitExceeded)
    } else {
        Ok(())
    }
}

fn entity_sort_key(namespace: AllocationNamespaceId, counter: u64) -> ([u8; 16], u64) {
    (namespace.persistent_bytes(), counter)
}

fn validate_envelope(bytes: &[u8]) -> Result<(), DecodeError> {
    validate_file_size(bytes.len() as u64)?;
    if bytes.len() < MAGIC.len() || bytes[..MAGIC.len()] != MAGIC {
        return Err(DecodeError::MalformedEncoding);
    }
    if bytes.len() < ENVELOPE_LEN {
        return Err(DecodeError::MalformedEncoding);
    }
    let major = u16::from_be_bytes([bytes[8], bytes[9]]);
    let minor = u16::from_be_bytes([bytes[10], bytes[11]]);
    if major != FORMAT_MAJOR || minor != FORMAT_MINOR {
        return Err(DecodeError::UnsupportedFormatVersion);
    }
    if bytes.len() == ENVELOPE_LEN {
        return Err(DecodeError::MalformedEncoding);
    }
    Ok(())
}

fn validate_file_size(len: u64) -> Result<(), DecodeError> {
    #[cfg(test)]
    LAST_FILE_SIZE_CHECK.with(|observed| observed.set(Some(len)));
    if len > MAX_FILE_SIZE as u64 {
        Err(DecodeError::ResourceLimitExceeded)
    } else {
        Ok(())
    }
}

#[cfg(test)]
thread_local! {
    static LAST_FILE_SIZE_CHECK: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}

fn validate_canonical_cbor(bytes: &[u8]) -> Result<(), DecodeError> {
    validate_canonical_input(&SliceCanonicalInput(bytes))
}

trait CanonicalInput {
    fn logical_len(&self) -> u64;
    fn byte(&self, position: u64) -> Option<u8>;
    fn chunk(&self, position: u64, maximum: usize) -> Option<&[u8]>;

    fn contains_range(&self, start: u64, len: u64) -> bool {
        start
            .checked_add(len)
            .is_some_and(|end| end <= self.logical_len())
    }
}

struct SliceCanonicalInput<'a>(&'a [u8]);

impl CanonicalInput for SliceCanonicalInput<'_> {
    fn logical_len(&self) -> u64 {
        self.0.len() as u64
    }

    fn byte(&self, position: u64) -> Option<u8> {
        usize::try_from(position)
            .ok()
            .and_then(|position| self.0.get(position))
            .copied()
    }

    fn chunk(&self, position: u64, maximum: usize) -> Option<&[u8]> {
        let start = usize::try_from(position).ok()?;
        let remaining = self.0.get(start..)?;
        Some(&remaining[..remaining.len().min(maximum)])
    }
}

fn validate_canonical_input(input: &impl CanonicalInput) -> Result<(), DecodeError> {
    let mut cursor = 0_u64;
    scan_item(input, &mut cursor, 1)?;
    if cursor != input.logical_len() {
        return Err(DecodeError::MalformedEncoding);
    }
    Ok(())
}

fn scan_item(input: &impl CanonicalInput, cursor: &mut u64, depth: u8) -> Result<(), DecodeError> {
    if depth > MAX_DEPTH {
        return Err(DecodeError::ResourceLimitExceeded);
    }
    let initial = input.byte(*cursor).ok_or(DecodeError::MalformedEncoding)?;
    *cursor = cursor
        .checked_add(1)
        .ok_or(DecodeError::ResourceLimitExceeded)?;
    let major = initial >> 5;
    let additional = initial & 0x1f;
    match major {
        0 | 1 => {
            let _ = scan_argument(input, cursor, additional)?;
        }
        2 | 3 => {
            let len = scan_argument(input, cursor, additional)?;
            validate_item_length(len)?;
            let end = cursor
                .checked_add(len)
                .ok_or(DecodeError::ResourceLimitExceeded)?;
            if !input.contains_range(*cursor, len) {
                return Err(DecodeError::MalformedEncoding);
            }
            if major == 3 {
                validate_utf8(input, *cursor, len)?;
            }
            *cursor = end;
        }
        4 => {
            let count = scan_argument(input, cursor, additional)?;
            validate_collection_count(count)?;
            for _ in 0..count {
                scan_item(input, cursor, depth + 1)?;
            }
        }
        7 if additional == 20 || additional == 21 => {}
        5..=7 => return Err(DecodeError::NonCanonicalEncoding),
        _ => return Err(DecodeError::NonCanonicalEncoding),
    }
    Ok(())
}

fn scan_argument(
    input: &impl CanonicalInput,
    cursor: &mut u64,
    additional: u8,
) -> Result<u64, DecodeError> {
    match additional {
        0..=23 => Ok(u64::from(additional)),
        24 => {
            let value = u64::from(read_exact::<1>(input, cursor)?[0]);
            if value < 24 {
                Err(DecodeError::NonCanonicalEncoding)
            } else {
                Ok(value)
            }
        }
        25 => {
            let value = u64::from(u16::from_be_bytes(read_exact::<2>(input, cursor)?));
            if value <= u64::from(u8::MAX) {
                Err(DecodeError::NonCanonicalEncoding)
            } else {
                Ok(value)
            }
        }
        26 => {
            let value = u64::from(u32::from_be_bytes(read_exact::<4>(input, cursor)?));
            if value <= u64::from(u16::MAX) {
                Err(DecodeError::NonCanonicalEncoding)
            } else {
                Ok(value)
            }
        }
        27 => {
            let value = u64::from_be_bytes(read_exact::<8>(input, cursor)?);
            if value <= u64::from(u32::MAX) {
                Err(DecodeError::NonCanonicalEncoding)
            } else {
                Ok(value)
            }
        }
        28..=31 => Err(DecodeError::NonCanonicalEncoding),
        _ => Err(DecodeError::NonCanonicalEncoding),
    }
}

fn read_exact<const N: usize>(
    input: &impl CanonicalInput,
    cursor: &mut u64,
) -> Result<[u8; N], DecodeError> {
    let mut result = [0_u8; N];
    for byte in &mut result {
        *byte = input.byte(*cursor).ok_or(DecodeError::MalformedEncoding)?;
        *cursor = cursor
            .checked_add(1)
            .ok_or(DecodeError::ResourceLimitExceeded)?;
    }
    Ok(result)
}

fn validate_item_length(len: u64) -> Result<(), DecodeError> {
    if len > MAX_ITEM_SIZE {
        Err(DecodeError::ResourceLimitExceeded)
    } else {
        Ok(())
    }
}

fn validate_collection_count(count: u64) -> Result<(), DecodeError> {
    if count > MAX_COLLECTION_COUNT {
        Err(DecodeError::ResourceLimitExceeded)
    } else {
        Ok(())
    }
}

fn validate_utf8(input: &impl CanonicalInput, start: u64, len: u64) -> Result<(), DecodeError> {
    const CHUNK_SIZE: usize = 64 * 1024;
    let end = start
        .checked_add(len)
        .ok_or(DecodeError::ResourceLimitExceeded)?;
    let mut cursor = start;
    let mut pending = [0_u8; 4];
    let mut pending_len = 0_usize;

    while cursor < end {
        let remaining = usize::try_from((end - cursor).min(CHUNK_SIZE as u64))
            .map_err(|_| DecodeError::ResourceLimitExceeded)?;
        let chunk = input
            .chunk(cursor, remaining)
            .filter(|chunk| !chunk.is_empty() && chunk.len() <= remaining)
            .ok_or(DecodeError::MalformedEncoding)?;
        let mut offset = 0_usize;

        if pending_len != 0 {
            let expected = utf8_sequence_len(pending[0]).ok_or(DecodeError::MalformedEncoding)?;
            let take = (expected - pending_len).min(chunk.len());
            pending[pending_len..pending_len + take].copy_from_slice(&chunk[..take]);
            pending_len += take;
            offset = take;
            if pending_len == expected {
                std::str::from_utf8(&pending[..expected])
                    .map_err(|_| DecodeError::MalformedEncoding)?;
                pending_len = 0;
            }
        }

        if pending_len == 0 && offset < chunk.len() {
            let remaining_chunk = &chunk[offset..];
            if let Err(error) = std::str::from_utf8(remaining_chunk) {
                if error.error_len().is_some() {
                    return Err(DecodeError::MalformedEncoding);
                }
                let suffix = &remaining_chunk[error.valid_up_to()..];
                if suffix.is_empty() || suffix.len() > 3 {
                    return Err(DecodeError::MalformedEncoding);
                }
                pending[..suffix.len()].copy_from_slice(suffix);
                pending_len = suffix.len();
            }
        }

        cursor = cursor
            .checked_add(chunk.len() as u64)
            .ok_or(DecodeError::ResourceLimitExceeded)?;
    }

    if pending_len == 0 {
        Ok(())
    } else {
        Err(DecodeError::MalformedEncoding)
    }
}

const fn utf8_sequence_len(first: u8) -> Option<usize> {
    match first {
        0x00..=0x7f => Some(1),
        0xc2..=0xdf => Some(2),
        0xe0..=0xef => Some(3),
        0xf0..=0xf4 => Some(4),
        _ => None,
    }
}

fn decode_payload(bytes: &[u8]) -> Result<PersistenceProgram, DecodeError> {
    let mut decoder = Decoder::new(bytes);
    fixed_array(&mut decoder, 5)?;
    let program_id = decode_program_id(&mut decoder)?;
    let revision_id = wire_u64(&mut decoder)?;
    let revision_counter_state = decode_revision_counter(&mut decoder)?;
    let (allocation_namespace_id, allocation_counter_state) =
        decode_allocation_authority(&mut decoder)?;
    let module_count = variable_array(&mut decoder)?;
    let mut modules = Vec::with_capacity(initial_capacity(module_count));
    let mut previous = None;
    let mut seen = std::collections::HashSet::new();
    for _ in 0..module_count {
        let module = decode_module(&mut decoder)?;
        check_collection_id(
            &mut previous,
            &mut seen,
            module.persistent_id().namespace_bytes(),
            module.persistent_id().counter(),
        )?;
        modules.push(module);
    }
    if decoder.position() != bytes.len() {
        return Err(DecodeError::MalformedEncoding);
    }
    Ok(PersistenceProgram::new(
        program_id,
        revision_id,
        revision_counter_state,
        allocation_namespace_id,
        allocation_counter_state,
        modules,
    ))
}

fn decode_module(decoder: &mut Decoder<'_>) -> Result<PersistenceModule, DecodeError> {
    fixed_array(decoder, 4)?;
    let id = decode_entity_id(decoder)?;
    let presentation = decode_presentation(decoder)?;
    let count = variable_array(decoder)?;
    let mut domain_types = Vec::with_capacity(initial_capacity(count));
    let mut previous = None;
    let mut seen = std::collections::HashSet::new();
    for _ in 0..count {
        let domain_type = decode_domain_type(decoder)?;
        check_collection_id(
            &mut previous,
            &mut seen,
            domain_type.persistent_id().namespace_bytes(),
            domain_type.persistent_id().counter(),
        )?;
        domain_types.push(domain_type);
    }
    let count = variable_array(decoder)?;
    let mut functions = Vec::with_capacity(initial_capacity(count));
    let mut previous = None;
    let mut seen = std::collections::HashSet::new();
    for _ in 0..count {
        let function = decode_function(decoder)?;
        check_collection_id(
            &mut previous,
            &mut seen,
            function.persistent_id().namespace_bytes(),
            function.persistent_id().counter(),
        )?;
        functions.push(function);
    }
    Ok(PersistenceModule::new(
        id,
        presentation,
        domain_types,
        functions,
    ))
}

fn decode_domain_type(decoder: &mut Decoder<'_>) -> Result<PersistenceDomainType, DecodeError> {
    fixed_array(decoder, 3)?;
    Ok(PersistenceDomainType::new(
        decode_entity_id(decoder)?,
        decode_intrinsic(decoder)?,
        decode_presentation(decoder)?,
    ))
}

fn decode_function(decoder: &mut Decoder<'_>) -> Result<PersistenceFunction, DecodeError> {
    fixed_array(decoder, 5)?;
    let id = decode_entity_id(decoder)?;
    let return_type = decode_persistence_value_type(decoder)?;
    let count = variable_array(decoder)?;
    let mut parameters = Vec::with_capacity(initial_capacity(count));
    for _ in 0..count {
        fixed_array(decoder, 3)?;
        parameters.push(PersistenceParameter::new(
            decode_entity_id(decoder)?,
            decode_persistence_value_type(decoder)?,
            decode_presentation(decoder)?,
        ));
    }
    let body = decode_body(decoder)?;
    let presentation = decode_presentation(decoder)?;
    Ok(PersistenceFunction::new(
        id,
        return_type,
        parameters,
        body,
        presentation,
    ))
}

fn decode_body(decoder: &mut Decoder<'_>) -> Result<Option<PersistenceBody>, DecodeError> {
    let length = array_len(decoder)?;
    let tag = wire_u64(decoder)?;
    match (tag, length) {
        (0, 1) => Ok(None),
        (1, 2) => {
            fixed_array(decoder, 2)?;
            let entry_block_id = decode_entity_id(decoder)?;
            let count = variable_array(decoder)?;
            let mut blocks = Vec::with_capacity(initial_capacity(count));
            let mut previous = None;
            let mut seen = std::collections::HashSet::new();
            for _ in 0..count {
                let block = decode_block(decoder)?;
                check_collection_id(
                    &mut previous,
                    &mut seen,
                    block.persistent_id().namespace_bytes(),
                    block.persistent_id().counter(),
                )?;
                blocks.push(block);
            }
            Ok(Some(PersistenceBody::new(entry_block_id, blocks)))
        }
        _ => Err(DecodeError::InvalidWireSchema),
    }
}

fn decode_block(decoder: &mut Decoder<'_>) -> Result<PersistenceBlock, DecodeError> {
    fixed_array(decoder, 4)?;
    let id = decode_entity_id(decoder)?;
    let count = variable_array(decoder)?;
    let mut expressions = Vec::with_capacity(initial_capacity(count));
    let mut previous = None;
    let mut seen = std::collections::HashSet::new();
    for _ in 0..count {
        fixed_array(decoder, 2)?;
        let expression_id = decode_entity_id(decoder)?;
        let kind = decode_expression_kind(decoder)?;
        check_collection_id(
            &mut previous,
            &mut seen,
            expression_id.namespace_bytes(),
            expression_id.counter(),
        )?;
        expressions.push(PersistenceExpression::new(expression_id, kind));
    }
    let count = variable_array(decoder)?;
    let mut effect_sequence = Vec::with_capacity(initial_capacity(count));
    for _ in 0..count {
        effect_sequence.push(decode_entity_id(decoder)?);
    }
    let terminator = decode_terminator(decoder)?;
    Ok(PersistenceBlock::new(
        id,
        expressions,
        effect_sequence,
        terminator,
    ))
}

fn decode_expression_kind(
    decoder: &mut Decoder<'_>,
) -> Result<PersistenceExpressionKind, DecodeError> {
    let length = array_len(decoder)?;
    let tag = wire_u64(decoder)?;
    let kind = match tag {
        0 if length == 2 => {
            let value = wire_i64(decoder)?;
            PersistenceExpressionKind::Int32Literal(
                i32::try_from(value).map_err(|_| DecodeError::InvalidWireSchema)?,
            )
        }
        1 if length == 2 => PersistenceExpressionKind::Int64Literal(wire_i64(decoder)?),
        2 if length == 2 => PersistenceExpressionKind::BoolLiteral(
            decoder.bool().map_err(|_| DecodeError::InvalidWireSchema)?,
        ),
        3 if length == 1 => PersistenceExpressionKind::UnitLiteral,
        4 if length == 2 => PersistenceExpressionKind::TextLiteral(
            decoder
                .str()
                .map_err(|_| DecodeError::InvalidWireSchema)?
                .to_owned(),
        ),
        5 if length == 2 => PersistenceExpressionKind::BytesLiteral(
            decoder
                .bytes()
                .map_err(|_| DecodeError::InvalidWireSchema)?
                .to_vec(),
        ),
        6 if length == 2 => {
            PersistenceExpressionKind::ParameterReference(decode_entity_id(decoder)?)
        }
        7..=17 if length == 3 => {
            let left = decode_entity_id(decoder)?;
            let right = decode_entity_id(decoder)?;
            match tag {
                7 => PersistenceExpressionKind::Add { left, right },
                8 => PersistenceExpressionKind::Subtract { left, right },
                9 => PersistenceExpressionKind::Multiply { left, right },
                10 => PersistenceExpressionKind::Divide { left, right },
                11 => PersistenceExpressionKind::Remainder { left, right },
                12 => PersistenceExpressionKind::Equal { left, right },
                13 => PersistenceExpressionKind::NotEqual { left, right },
                14 => PersistenceExpressionKind::LessThan { left, right },
                15 => PersistenceExpressionKind::LessThanOrEqual { left, right },
                16 => PersistenceExpressionKind::GreaterThan { left, right },
                17 => PersistenceExpressionKind::GreaterThanOrEqual { left, right },
                _ => unreachable!(),
            }
        }
        18 if length == 3 => {
            let target = decode_entity_id(decoder)?;
            let count = variable_array(decoder)?;
            let mut arguments = Vec::with_capacity(initial_capacity(count));
            for _ in 0..count {
                arguments.push(decode_entity_id(decoder)?);
            }
            PersistenceExpressionKind::Call { target, arguments }
        }
        19 if length == 3 => PersistenceExpressionKind::DomainConstruct {
            type_id: decode_entity_id(decoder)?,
            value: decode_entity_id(decoder)?,
        },
        20 if length == 2 => PersistenceExpressionKind::DomainProject {
            value: decode_entity_id(decoder)?,
        },
        _ => return Err(DecodeError::InvalidWireSchema),
    };
    Ok(kind)
}

fn decode_terminator(decoder: &mut Decoder<'_>) -> Result<PersistenceTerminator, DecodeError> {
    let length = array_len(decoder)?;
    let tag = wire_u64(decoder)?;
    match (tag, length) {
        (0, 2) => Ok(PersistenceTerminator::Return {
            expression: decode_entity_id(decoder)?,
        }),
        (1, 4) => Ok(PersistenceTerminator::Branch {
            condition: decode_entity_id(decoder)?,
            true_block: decode_entity_id(decoder)?,
            false_block: decode_entity_id(decoder)?,
        }),
        _ => Err(DecodeError::InvalidWireSchema),
    }
}

fn decode_presentation(decoder: &mut Decoder<'_>) -> Result<PersistencePresentation, DecodeError> {
    fixed_array(decoder, 2)?;
    Ok(PersistencePresentation::new(
        decode_string_option(decoder)?,
        decode_string_option(decoder)?,
    ))
}

fn decode_string_option(decoder: &mut Decoder<'_>) -> Result<Option<String>, DecodeError> {
    let length = array_len(decoder)?;
    let tag = wire_u64(decoder)?;
    match (tag, length) {
        (0, 1) => Ok(None),
        (1, 2) => Ok(Some(
            decoder
                .str()
                .map_err(|_| DecodeError::InvalidWireSchema)?
                .to_owned(),
        )),
        _ => Err(DecodeError::InvalidWireSchema),
    }
}

fn decode_persistence_value_type(
    decoder: &mut Decoder<'_>,
) -> Result<PersistenceValueType, DecodeError> {
    fixed_array(decoder, 2)?;
    match wire_u64(decoder)? {
        0 => Ok(PersistenceValueType::Intrinsic(decode_intrinsic(decoder)?)),
        1 => Ok(PersistenceValueType::Domain(decode_entity_id(decoder)?)),
        _ => Err(DecodeError::InvalidWireSchema),
    }
}

fn decode_intrinsic(decoder: &mut Decoder<'_>) -> Result<IntrinsicType, DecodeError> {
    match wire_u64(decoder)? {
        0 => Ok(IntrinsicType::Int32),
        1 => Ok(IntrinsicType::Int64),
        2 => Ok(IntrinsicType::Bool),
        3 => Ok(IntrinsicType::Unit),
        4 => Ok(IntrinsicType::Text),
        5 => Ok(IntrinsicType::Bytes),
        _ => Err(DecodeError::InvalidWireSchema),
    }
}

fn decode_revision_counter(decoder: &mut Decoder<'_>) -> Result<RevisionCounterState, DecodeError> {
    let length = array_len(decoder)?;
    match (wire_u64(decoder)?, length) {
        (0, 2) => {
            let next = wire_u64(decoder)?;
            if next == 0 {
                Err(DecodeError::InvalidWireSchema)
            } else {
                Ok(RevisionCounterState::Available(next))
            }
        }
        (1, 1) => Ok(RevisionCounterState::Exhausted),
        _ => Err(DecodeError::InvalidWireSchema),
    }
}

fn decode_allocation_authority(
    decoder: &mut Decoder<'_>,
) -> Result<([u8; 16], AllocationCounterState), DecodeError> {
    fixed_array(decoder, 2)?;
    let namespace = decode_namespace(decoder)?;
    let length = array_len(decoder)?;
    let state = match (wire_u64(decoder)?, length) {
        (0, 2) => AllocationCounterState::Available(wire_u64(decoder)?),
        (1, 1) => AllocationCounterState::Exhausted,
        _ => return Err(DecodeError::InvalidWireSchema),
    };
    Ok((namespace, state))
}

fn decode_program_id(decoder: &mut Decoder<'_>) -> Result<[u8; 16], DecodeError> {
    fixed_bytes_16(decoder)
}

fn decode_namespace(decoder: &mut Decoder<'_>) -> Result<[u8; 16], DecodeError> {
    fixed_bytes_16(decoder)
}

fn decode_entity_id(decoder: &mut Decoder<'_>) -> Result<PersistenceEntityId, DecodeError> {
    fixed_array(decoder, 2)?;
    Ok(PersistenceEntityId::new(
        decode_namespace(decoder)?,
        wire_u64(decoder)?,
    ))
}

fn fixed_bytes_16(decoder: &mut Decoder<'_>) -> Result<[u8; 16], DecodeError> {
    let bytes = decoder
        .bytes()
        .map_err(|_| DecodeError::InvalidWireSchema)?;
    bytes.try_into().map_err(|_| DecodeError::InvalidWireSchema)
}

fn wire_u64(decoder: &mut Decoder<'_>) -> Result<u64, DecodeError> {
    decoder.u64().map_err(|_| DecodeError::InvalidWireSchema)
}

fn wire_i64(decoder: &mut Decoder<'_>) -> Result<i64, DecodeError> {
    decoder.i64().map_err(|_| DecodeError::InvalidWireSchema)
}

fn array_len(decoder: &mut Decoder<'_>) -> Result<u64, DecodeError> {
    decoder
        .array()
        .map_err(|_| DecodeError::InvalidWireSchema)?
        .ok_or(DecodeError::NonCanonicalEncoding)
}

fn fixed_array(decoder: &mut Decoder<'_>, expected: u64) -> Result<(), DecodeError> {
    if array_len(decoder)? == expected {
        Ok(())
    } else {
        Err(DecodeError::InvalidWireSchema)
    }
}

fn variable_array(decoder: &mut Decoder<'_>) -> Result<usize, DecodeError> {
    let count = array_len(decoder)?;
    validate_collection_count(count)?;
    usize::try_from(count).map_err(|_| DecodeError::ResourceLimitExceeded)
}

fn initial_capacity(count: usize) -> usize {
    count.min(4096)
}

fn check_collection_id(
    previous: &mut Option<([u8; 16], u64)>,
    seen: &mut std::collections::HashSet<([u8; 16], u64)>,
    namespace: [u8; 16],
    counter: u64,
) -> Result<(), DecodeError> {
    let current = (namespace, counter);
    if !seen.insert(current) {
        return Err(DecodeError::StructurallyInvalidProgram);
    }
    if previous
        .as_ref()
        .is_some_and(|prior| prior.cmp(&current) != Ordering::Less)
    {
        return Err(DecodeError::NonCanonicalEncoding);
    }
    *previous = Some(current);
    Ok(())
}

#[cfg(test)]
mod wire_fragment_tests {
    use super::*;

    use std::cell::Cell;

    fn namespace() -> [u8; 16] {
        [
            0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d,
            0x1e, 0x1f,
        ]
    }

    enum VirtualSegmentContents {
        Literal(Vec<u8>),
        RepeatedByte(Vec<u8>),
        RepeatedPattern(Vec<u8>),
    }

    struct VirtualSegment {
        start: u64,
        len: u64,
        contents: VirtualSegmentContents,
    }

    struct VirtualCanonicalInput {
        segments: Vec<VirtualSegment>,
        len: u64,
        cached_segment: Cell<usize>,
    }

    impl VirtualCanonicalInput {
        fn segment_at(&self, position: u64) -> Option<&VirtualSegment> {
            if position >= self.len {
                return None;
            }
            let cached = self.cached_segment.get();
            if let Some(segment) = self.segments.get(cached)
                && position >= segment.start
                && position < segment.start + segment.len
            {
                return Some(segment);
            }
            let index = self
                .segments
                .partition_point(|segment| segment.start <= position)
                .checked_sub(1)?;
            self.cached_segment.set(index);
            self.segments.get(index)
        }

        fn materialize(&self) -> Vec<u8> {
            let capacity = usize::try_from(self.len).expect("small fixture length");
            let mut bytes = Vec::with_capacity(capacity);
            let mut position = 0_u64;
            while position < self.len {
                let chunk = self.chunk(position, 64 * 1024).expect("complete fixture");
                bytes.extend_from_slice(chunk);
                position += chunk.len() as u64;
            }
            bytes
        }
    }

    impl CanonicalInput for VirtualCanonicalInput {
        fn logical_len(&self) -> u64 {
            self.len
        }

        fn byte(&self, position: u64) -> Option<u8> {
            let segment = self.segment_at(position)?;
            let offset = position - segment.start;
            match &segment.contents {
                VirtualSegmentContents::Literal(bytes) => {
                    bytes.get(usize::try_from(offset).ok()?).copied()
                }
                VirtualSegmentContents::RepeatedByte(chunk) => chunk.first().copied(),
                VirtualSegmentContents::RepeatedPattern(pattern) => {
                    let offset = usize::try_from(offset % pattern.len() as u64).ok()?;
                    pattern.get(offset).copied()
                }
            }
        }

        fn chunk(&self, position: u64, maximum: usize) -> Option<&[u8]> {
            if maximum == 0 {
                return None;
            }
            let segment = self.segment_at(position)?;
            let offset = position - segment.start;
            let segment_remaining =
                usize::try_from((segment.len - offset).min(maximum as u64)).ok()?;
            match &segment.contents {
                VirtualSegmentContents::Literal(bytes) => {
                    let offset = usize::try_from(offset).ok()?;
                    bytes.get(offset..offset + segment_remaining)
                }
                VirtualSegmentContents::RepeatedByte(chunk) => {
                    chunk.get(..segment_remaining.min(chunk.len()))
                }
                VirtualSegmentContents::RepeatedPattern(pattern) => {
                    let offset = usize::try_from(offset % pattern.len() as u64).ok()?;
                    let len = segment_remaining.min(pattern.len() - offset);
                    pattern.get(offset..offset + len)
                }
            }
        }
    }

    #[derive(Default)]
    struct VirtualFixtureBuilder {
        segments: Vec<VirtualSegment>,
        len: u64,
    }

    impl VirtualFixtureBuilder {
        fn literal(&mut self, bytes: impl Into<Vec<u8>>) {
            let bytes = bytes.into();
            if bytes.is_empty() {
                return;
            }
            let len = bytes.len() as u64;
            self.segments.push(VirtualSegment {
                start: self.len,
                len,
                contents: VirtualSegmentContents::Literal(bytes),
            });
            self.len += len;
        }

        fn repeated_byte(&mut self, byte: u8, len: u64) {
            if len == 0 {
                return;
            }
            self.segments.push(VirtualSegment {
                start: self.len,
                len,
                contents: VirtualSegmentContents::RepeatedByte(vec![byte; 64 * 1024]),
            });
            self.len += len;
        }

        fn repeated_pattern(&mut self, pattern: Vec<u8>, repetitions: u64) {
            if repetitions == 0 {
                return;
            }
            let len = (pattern.len() as u64)
                .checked_mul(repetitions)
                .expect("bounded conformance fixture");
            self.segments.push(VirtualSegment {
                start: self.len,
                len,
                contents: VirtualSegmentContents::RepeatedPattern(pattern),
            });
            self.len += len;
        }

        fn finish(self) -> VirtualCanonicalInput {
            VirtualCanonicalInput {
                segments: self.segments,
                len: self.len,
                cached_segment: Cell::new(0),
            }
        }
    }

    fn cbor_head(major: u8, value: u64) -> Vec<u8> {
        let mut bytes = Vec::new();
        match value {
            0..=23 => bytes.push((major << 5) | value as u8),
            24..=255 => bytes.extend([(major << 5) | 24, value as u8]),
            256..=65_535 => {
                bytes.push((major << 5) | 25);
                bytes.extend((value as u16).to_be_bytes());
            }
            65_536..=4_294_967_295 => {
                bytes.push((major << 5) | 26);
                bytes.extend((value as u32).to_be_bytes());
            }
            _ => {
                bytes.push((major << 5) | 27);
                bytes.extend(value.to_be_bytes());
            }
        }
        bytes
    }

    fn fixture_array(builder: &mut VirtualFixtureBuilder, count: u64) {
        builder.literal(cbor_head(4, count));
    }

    fn fixture_u64(builder: &mut VirtualFixtureBuilder, value: u64) {
        builder.literal(cbor_head(0, value));
    }

    fn fixture_bytes(builder: &mut VirtualFixtureBuilder, byte: u8, len: u64) {
        builder.literal(cbor_head(2, len));
        builder.repeated_byte(byte, len);
    }

    fn fixture_text(builder: &mut VirtualFixtureBuilder, byte: u8, len: u64) {
        builder.literal(cbor_head(3, len));
        builder.repeated_byte(byte, len);
    }

    fn fixture_id(builder: &mut VirtualFixtureBuilder, counter: u64) {
        fixture_array(builder, 2);
        fixture_bytes(builder, 0x52, 16);
        fixture_u64(builder, counter);
    }

    fn fixture_absent_presentation(builder: &mut VirtualFixtureBuilder) {
        fixture_array(builder, 2);
        fixture_array(builder, 1);
        fixture_u64(builder, 0);
        fixture_array(builder, 1);
        fixture_u64(builder, 0);
    }

    fn fixture_program_prefix(builder: &mut VirtualFixtureBuilder, allocation_next: u64) {
        fixture_array(builder, 5);
        fixture_bytes(builder, 0x51, 16);
        fixture_u64(builder, 1);
        fixture_array(builder, 2);
        fixture_u64(builder, 0);
        fixture_u64(builder, 2);
        fixture_array(builder, 2);
        fixture_bytes(builder, 0x52, 16);
        fixture_array(builder, 2);
        fixture_u64(builder, 0);
        fixture_u64(builder, allocation_next);
    }

    fn literal_program(kind_tag: u64, content_lengths: &[u64]) -> VirtualCanonicalInput {
        assert!(!content_lengths.is_empty());
        let mut builder = VirtualFixtureBuilder::default();
        fixture_program_prefix(&mut builder, 3 + content_lengths.len() as u64);
        fixture_array(&mut builder, 1);
        fixture_array(&mut builder, 4);
        fixture_id(&mut builder, 0);
        fixture_absent_presentation(&mut builder);
        fixture_array(&mut builder, 0);
        fixture_array(&mut builder, 1);
        fixture_array(&mut builder, 5);
        fixture_id(&mut builder, 1);
        fixture_array(&mut builder, 2);
        fixture_u64(&mut builder, 0);
        fixture_u64(&mut builder, kind_tag);
        fixture_array(&mut builder, 0);
        fixture_array(&mut builder, 2);
        fixture_u64(&mut builder, 1);
        fixture_array(&mut builder, 2);
        fixture_id(&mut builder, 2);
        fixture_array(&mut builder, 1);
        fixture_array(&mut builder, 4);
        fixture_id(&mut builder, 2);
        fixture_array(&mut builder, content_lengths.len() as u64);
        for (index, &content_len) in content_lengths.iter().enumerate() {
            fixture_array(&mut builder, 2);
            fixture_id(&mut builder, 3 + index as u64);
            fixture_array(&mut builder, 2);
            fixture_u64(&mut builder, kind_tag);
            if kind_tag == 4 {
                fixture_text(&mut builder, b'a', content_len);
            } else {
                fixture_bytes(&mut builder, 0xa5, content_len);
            }
        }
        fixture_array(&mut builder, 0);
        fixture_array(&mut builder, 2);
        fixture_u64(&mut builder, 0);
        fixture_id(&mut builder, 2 + content_lengths.len() as u64);
        fixture_absent_presentation(&mut builder);
        builder.finish()
    }

    fn encoded_fixture_id(counter: u64) -> Vec<u8> {
        let mut builder = VirtualFixtureBuilder::default();
        fixture_id(&mut builder, counter);
        builder.finish().materialize()
    }

    fn call_arguments_program(argument_count: u64) -> VirtualCanonicalInput {
        let mut builder = VirtualFixtureBuilder::default();
        fixture_program_prefix(&mut builder, 6);
        fixture_array(&mut builder, 1);
        fixture_array(&mut builder, 4);
        fixture_id(&mut builder, 0);
        fixture_absent_presentation(&mut builder);
        fixture_array(&mut builder, 0);
        fixture_array(&mut builder, 2);

        fixture_array(&mut builder, 5);
        fixture_id(&mut builder, 1);
        fixture_array(&mut builder, 2);
        fixture_u64(&mut builder, 0);
        fixture_u64(&mut builder, 3);
        fixture_array(&mut builder, 0);
        fixture_array(&mut builder, 1);
        fixture_u64(&mut builder, 0);
        fixture_absent_presentation(&mut builder);

        fixture_array(&mut builder, 5);
        fixture_id(&mut builder, 2);
        fixture_array(&mut builder, 2);
        fixture_u64(&mut builder, 0);
        fixture_u64(&mut builder, 3);
        fixture_array(&mut builder, 0);
        fixture_array(&mut builder, 2);
        fixture_u64(&mut builder, 1);
        fixture_array(&mut builder, 2);
        fixture_id(&mut builder, 3);
        fixture_array(&mut builder, 1);
        fixture_array(&mut builder, 4);
        fixture_id(&mut builder, 3);
        fixture_array(&mut builder, 2);

        fixture_array(&mut builder, 2);
        fixture_id(&mut builder, 4);
        fixture_array(&mut builder, 1);
        fixture_u64(&mut builder, 3);

        fixture_array(&mut builder, 2);
        fixture_id(&mut builder, 5);
        fixture_array(&mut builder, 3);
        fixture_u64(&mut builder, 18);
        fixture_id(&mut builder, 1);
        fixture_array(&mut builder, argument_count);
        builder.repeated_pattern(encoded_fixture_id(4), argument_count);

        fixture_array(&mut builder, 1);
        fixture_id(&mut builder, 5);
        fixture_array(&mut builder, 2);
        fixture_u64(&mut builder, 0);
        fixture_id(&mut builder, 5);
        fixture_absent_presentation(&mut builder);
        builder.finish()
    }

    fn fixture_file(payload: &VirtualCanonicalInput) -> Vec<u8> {
        let mut bytes = Vec::from(MAGIC);
        bytes.extend(FORMAT_MAJOR.to_be_bytes());
        bytes.extend(FORMAT_MINOR.to_be_bytes());
        bytes.extend(payload.materialize());
        bytes
    }

    // AR-SER-005, AR-SER-037; MNIR-SER-101.
    #[test]
    fn typed_id_fragment_matches_normative_bytes() {
        let mut encoder = Encoder::new(LimitedVec::default());
        encode_raw_entity_id(&mut encoder, namespace(), 24).unwrap();
        assert_eq!(
            encoder.into_writer().bytes,
            [
                0x82, 0x50, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
                0x1c, 0x1d, 0x1e, 0x1f, 0x18, 0x18,
            ]
        );
    }

    fn expected_id(suffix: &[u8]) -> Vec<u8> {
        let mut expected = vec![
            0x82, 0x50, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f,
        ];
        expected.extend_from_slice(suffix);
        expected
    }

    // AR-SER-005 and AR-SER-037: counter encodings are compared with fixed
    // independently written bytes at every required CBOR width boundary.
    #[test]
    fn every_identifier_counter_boundary_has_fixed_bytes() {
        let cases: &[(u64, &[u8])] = &[
            (0, &[0x00]),
            (1, &[0x01]),
            (23, &[0x17]),
            (24, &[0x18, 0x18]),
            (255, &[0x18, 0xff]),
            (256, &[0x19, 0x01, 0x00]),
            (
                u64::from(u32::MAX) + 1,
                &[0x1b, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
            ),
            (
                u64::MAX,
                &[0x1b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff],
            ),
        ];
        for &(counter, suffix) in cases {
            let mut encoder = Encoder::new(LimitedVec::default());
            encode_raw_entity_id(&mut encoder, namespace(), counter).unwrap();
            assert_eq!(encoder.into_writer().bytes, expected_id(suffix));
        }

        let mut revision = Encoder::new(LimitedVec::default());
        revision.u64(u64::MAX).unwrap();
        assert_eq!(
            revision.into_writer().bytes,
            [0x1b, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]
        );
    }

    // AR-SER-005: the production encoder for every typed category uses the
    // exact category-neutral persistent pair, with no Rust discriminant.
    #[test]
    fn all_six_typed_id_categories_have_fixed_bytes() {
        let candidate = PersistenceProgram::new(
            [0x20; 16],
            1,
            RevisionCounterState::Available(2),
            namespace(),
            AllocationCounterState::Available(257),
            vec![PersistenceModule::new(
                PersistenceEntityId::new(namespace(), 0),
                PersistencePresentation::new(None, None),
                vec![PersistenceDomainType::new(
                    PersistenceEntityId::new(namespace(), 1),
                    IntrinsicType::Int32,
                    PersistencePresentation::new(None, None),
                )],
                vec![PersistenceFunction::new(
                    PersistenceEntityId::new(namespace(), 23),
                    PersistenceValueType::Intrinsic(IntrinsicType::Unit),
                    vec![PersistenceParameter::new(
                        PersistenceEntityId::new(namespace(), 24),
                        PersistenceValueType::Intrinsic(IntrinsicType::Unit),
                        PersistencePresentation::new(None, None),
                    )],
                    Some(PersistenceBody::new(
                        PersistenceEntityId::new(namespace(), 255),
                        vec![PersistenceBlock::new(
                            PersistenceEntityId::new(namespace(), 255),
                            vec![PersistenceExpression::new(
                                PersistenceEntityId::new(namespace(), 256),
                                PersistenceExpressionKind::UnitLiteral,
                            )],
                            Vec::new(),
                            PersistenceTerminator::Return {
                                expression: PersistenceEntityId::new(namespace(), 256),
                            },
                        )],
                    )),
                    PersistencePresentation::new(None, None),
                )],
            )],
        );
        let validated = MnirProgram::validate_persistence(candidate).unwrap();
        let module = validated.modules().next().unwrap();
        let domain = module.domain_types().next().unwrap();
        let function = module.functions().next().unwrap();
        let parameter = &function.parameters()[0];
        let block = function.body().unwrap().blocks().next().unwrap();
        let expression = block.expressions().next().unwrap();

        macro_rules! encoded {
            ($call:expr) => {{
                let mut encoder = Encoder::new(LimitedVec::default());
                $call(&mut encoder).unwrap();
                encoder.into_writer().bytes
            }};
        }
        let actual = vec![
            encoded!(|encoder| encode_module_id(encoder, module.id())),
            encoded!(|encoder| encode_type_id(encoder, domain.id())),
            encoded!(|encoder| encode_function_id(encoder, function.id())),
            encoded!(|encoder| encode_parameter_id(encoder, parameter.id())),
            encoded!(|encoder| encode_block_id(encoder, block.id())),
            encoded!(|encoder| encode_expression_id(encoder, expression.id())),
        ];
        assert_eq!(
            actual,
            vec![
                expected_id(&[0x00]),
                expected_id(&[0x01]),
                expected_id(&[0x17]),
                expected_id(&[0x18, 0x18]),
                expected_id(&[0x18, 0xff]),
                expected_id(&[0x19, 0x01, 0x00]),
            ]
        );

        let namespace_candidate = |program, namespace| {
            MnirProgram::validate_persistence(PersistenceProgram::new(
                [program; 16],
                1,
                RevisionCounterState::Available(2),
                [namespace; 16],
                AllocationCounterState::Available(0),
                Vec::new(),
            ))
            .unwrap()
            .allocation_namespace_id()
        };
        let lower = namespace_candidate(0x30, 0x00);
        let higher = namespace_candidate(0x31, 0x01);
        assert!(entity_sort_key(lower, u64::MAX) < entity_sort_key(higher, 0));
        assert!(
            entity_sort_key(validated.allocation_namespace_id(), 0)
                < entity_sort_key(validated.allocation_namespace_id(), u64::MAX)
        );
    }

    // AR-SER-020, AR-SER-022, AR-SER-037; MNIR-SER-102.
    #[test]
    fn allocator_fragments_match_normative_bytes() {
        let mut available = Encoder::new(LimitedVec::default());
        encode_raw_allocation_authority(
            &mut available,
            namespace(),
            AllocationCounterState::Available(24),
        )
        .unwrap();
        let mut expected = vec![0x82, 0x50];
        expected.extend_from_slice(&namespace());
        expected.extend_from_slice(&[0x82, 0x00, 0x18, 0x18]);
        assert_eq!(available.into_writer().bytes, expected);

        let mut exhausted = Encoder::new(LimitedVec::default());
        encode_raw_allocation_authority(
            &mut exhausted,
            namespace(),
            AllocationCounterState::Exhausted,
        )
        .unwrap();
        let mut expected = vec![0x82, 0x50];
        expected.extend_from_slice(&namespace());
        expected.extend_from_slice(&[0x81, 0x01]);
        assert_eq!(exhausted.into_writer().bytes, expected);
    }

    // AR-SER-008, AR-SER-010, AR-SER-037; MNIR-SER-103.
    #[test]
    fn presentation_text_bytes_and_signed_fragments_are_exact() {
        let mut presentation = Encoder::new(LimitedVec::default());
        presentation.array(2).unwrap();
        encode_string_option(&mut presentation, None).unwrap();
        encode_string_option(&mut presentation, None).unwrap();
        assert_eq!(
            presentation.into_writer().bytes,
            [0x82, 0x81, 0x00, 0x81, 0x00]
        );

        let mut text = Encoder::new(LimitedVec::default());
        text.str("Aé").unwrap();
        assert_eq!(text.into_writer().bytes, [0x63, 0x41, 0xc3, 0xa9]);

        let mut bytes = Encoder::new(LimitedVec::default());
        bytes.bytes(&[0, 0xff]).unwrap();
        assert_eq!(bytes.into_writer().bytes, [0x42, 0x00, 0xff]);

        for (kind, expected) in [
            (ExpressionKind::Int32Literal(0), vec![0x82, 0x00, 0x00]),
            (ExpressionKind::Int32Literal(-1), vec![0x82, 0x00, 0x20]),
            (
                ExpressionKind::Int32Literal(i32::MIN),
                vec![0x82, 0x00, 0x3a, 0x7f, 0xff, 0xff, 0xff],
            ),
        ] {
            let mut encoder = Encoder::new(LimitedVec::default());
            encode_expression_kind(&mut encoder, &kind).unwrap();
            assert_eq!(encoder.into_writer().bytes, expected);
        }
    }

    // AR-SER-037: representative ValueType, reference-bearing Expression,
    // and terminator fragments are checked against independently specified
    // complete bytes, not only their leading schema tags.
    #[test]
    fn reference_bearing_schema_families_have_fixed_golden_fragments() {
        let id = |counter| PersistenceEntityId::new(namespace(), counter);
        let candidate = PersistenceProgram::new(
            [0x40; 16],
            1,
            RevisionCounterState::Available(2),
            namespace(),
            AllocationCounterState::Available(22),
            vec![PersistenceModule::new(
                id(0),
                PersistencePresentation::new(None, None),
                vec![PersistenceDomainType::new(
                    id(1),
                    IntrinsicType::Int32,
                    PersistencePresentation::new(None, None),
                )],
                vec![
                    PersistenceFunction::new(
                        id(2),
                        PersistenceValueType::Intrinsic(IntrinsicType::Unit),
                        Vec::new(),
                        None,
                        PersistencePresentation::new(None, None),
                    ),
                    PersistenceFunction::new(
                        id(3),
                        PersistenceValueType::Domain(id(1)),
                        vec![PersistenceParameter::new(
                            id(4),
                            PersistenceValueType::Intrinsic(IntrinsicType::Int32),
                            PersistencePresentation::new(None, None),
                        )],
                        Some(PersistenceBody::new(
                            id(5),
                            vec![
                                PersistenceBlock::new(
                                    id(5),
                                    vec![
                                        PersistenceExpression::new(
                                            id(6),
                                            PersistenceExpressionKind::Int32Literal(1),
                                        ),
                                        PersistenceExpression::new(
                                            id(7),
                                            PersistenceExpressionKind::ParameterReference(id(4)),
                                        ),
                                        PersistenceExpression::new(
                                            id(8),
                                            PersistenceExpressionKind::Add {
                                                left: id(6),
                                                right: id(7),
                                            },
                                        ),
                                        PersistenceExpression::new(
                                            id(9),
                                            PersistenceExpressionKind::Call {
                                                target: id(2),
                                                arguments: vec![id(8)],
                                            },
                                        ),
                                        PersistenceExpression::new(
                                            id(10),
                                            PersistenceExpressionKind::DomainConstruct {
                                                type_id: id(1),
                                                value: id(8),
                                            },
                                        ),
                                        PersistenceExpression::new(
                                            id(11),
                                            PersistenceExpressionKind::DomainProject {
                                                value: id(10),
                                            },
                                        ),
                                        PersistenceExpression::new(
                                            id(12),
                                            PersistenceExpressionKind::BoolLiteral(true),
                                        ),
                                    ],
                                    vec![id(9)],
                                    PersistenceTerminator::Branch {
                                        condition: id(12),
                                        true_block: id(20),
                                        false_block: id(20),
                                    },
                                ),
                                PersistenceBlock::new(
                                    id(20),
                                    vec![PersistenceExpression::new(
                                        id(21),
                                        PersistenceExpressionKind::UnitLiteral,
                                    )],
                                    Vec::new(),
                                    PersistenceTerminator::Return { expression: id(21) },
                                ),
                            ],
                        )),
                        PersistencePresentation::new(None, None),
                    ),
                ],
            )],
        );
        let validated = MnirProgram::validate_persistence(candidate).unwrap();
        let module = validated.modules().next().unwrap();
        let function = module
            .functions()
            .find(|function| function.id().counter() == 3)
            .unwrap();
        let body = function.body().unwrap();
        let entry = body
            .blocks()
            .find(|block| block.id().counter() == 5)
            .unwrap();
        let exit = body
            .blocks()
            .find(|block| block.id().counter() == 20)
            .unwrap();
        let kind = |counter| {
            entry
                .expressions()
                .find(|expression| expression.id().counter() == counter)
                .unwrap()
                .kind()
        };
        let encode_kind = |kind: &ExpressionKind| {
            let mut encoder = Encoder::new(LimitedVec::default());
            encode_expression_kind(&mut encoder, kind).unwrap();
            encoder.into_writer().bytes
        };
        let concat = |head: &[u8], id_suffixes: &[&[u8]]| {
            let mut bytes = head.to_vec();
            for suffix in id_suffixes {
                bytes.extend(expected_id(suffix));
            }
            bytes
        };

        let mut value_type = Encoder::new(LimitedVec::default());
        encode_value_type(&mut value_type, *function.return_type()).unwrap();
        assert_eq!(
            value_type.into_writer().bytes,
            concat(&[0x82, 0x01], &[&[0x01]])
        );
        assert_eq!(encode_kind(kind(7)), concat(&[0x82, 0x06], &[&[0x04]]));
        assert_eq!(
            encode_kind(kind(8)),
            concat(&[0x83, 0x07], &[&[0x06], &[0x07]])
        );
        let mut call = concat(&[0x83, 0x12], &[&[0x02]]);
        call.push(0x81);
        call.extend(expected_id(&[0x08]));
        assert_eq!(encode_kind(kind(9)), call);
        assert_eq!(
            encode_kind(kind(10)),
            concat(&[0x83, 0x13], &[&[0x01], &[0x08]])
        );
        assert_eq!(encode_kind(kind(11)), concat(&[0x82, 0x14], &[&[0x0a]]));

        let mut branch = Encoder::new(LimitedVec::default());
        encode_terminator(&mut branch, *entry.terminator().unwrap()).unwrap();
        assert_eq!(
            branch.into_writer().bytes,
            concat(&[0x84, 0x01], &[&[0x0c], &[0x14], &[0x14]])
        );
        let mut returned = Encoder::new(LimitedVec::default());
        encode_terminator(&mut returned, *exit.terminator().unwrap()).unwrap();
        assert_eq!(
            returned.into_writer().bytes,
            concat(&[0x82, 0x00], &[&[0x15]])
        );

        for (intrinsic, expected) in [
            (IntrinsicType::Int32, [0x82, 0x00, 0x00]),
            (IntrinsicType::Int64, [0x82, 0x00, 0x01]),
            (IntrinsicType::Bool, [0x82, 0x00, 0x02]),
            (IntrinsicType::Unit, [0x82, 0x00, 0x03]),
            (IntrinsicType::Text, [0x82, 0x00, 0x04]),
            (IntrinsicType::Bytes, [0x82, 0x00, 0x05]),
        ] {
            let mut encoder = Encoder::new(LimitedVec::default());
            encode_value_type(&mut encoder, ValueType::Intrinsic(intrinsic)).unwrap();
            assert_eq!(encoder.into_writer().bytes, expected);
        }
    }

    // AR-SER-031/-032: the exact production guards remain inclusive and are
    // shared by byte-backed decoding and virtual conformance input.
    #[test]
    fn exact_resource_guards_accept_max_and_reject_max_plus_one() {
        assert_eq!(validate_file_size(MAX_FILE_SIZE as u64), Ok(()));
        assert_eq!(
            validate_file_size(MAX_FILE_SIZE as u64 + 1),
            Err(DecodeError::ResourceLimitExceeded)
        );
        assert_eq!(validate_item_length(MAX_ITEM_SIZE), Ok(()));
        assert_eq!(
            validate_item_length(MAX_ITEM_SIZE + 1),
            Err(DecodeError::ResourceLimitExceeded)
        );
        assert_eq!(validate_collection_count(MAX_COLLECTION_COUNT), Ok(()));
        assert_eq!(
            validate_collection_count(MAX_COLLECTION_COUNT + 1),
            Err(DecodeError::ResourceLimitExceeded)
        );
        assert_eq!(check_item_len(MAX_ITEM_SIZE as usize), Ok(()));
        assert_eq!(
            check_item_len(MAX_ITEM_SIZE as usize + 1),
            Err(EncodeError::ResourceLimitExceeded)
        );
        let mut encoder = Encoder::new(LimitedVec::default());
        assert!(encode_count(&mut encoder, MAX_COLLECTION_COUNT as usize).is_ok());
        assert_eq!(
            encode_count(&mut encoder, MAX_COLLECTION_COUNT as usize + 1),
            Err(EncodeError::ResourceLimitExceeded)
        );
    }

    // AR-SER-031/-032 compositional part B: ordinary public byte-backed
    // decoding invokes the exact instrumented production file guard and
    // accepts the same schema shapes used by the maximum virtual fixtures.
    #[test]
    fn public_decode_uses_production_guard_and_virtual_fixture_schema_shapes() {
        let fixtures = [
            literal_program(4, &[4]),
            literal_program(5, &[4]),
            literal_program(5, &[1, 2, 3, 4]),
            call_arguments_program(2),
        ];
        for payload in fixtures {
            let bytes = fixture_file(&payload);
            LAST_FILE_SIZE_CHECK.with(|observed| observed.set(None));
            let decoded = decode(&bytes).unwrap();
            assert_eq!(decoded.module_count(), 1);
            LAST_FILE_SIZE_CHECK.with(|observed| {
                assert_eq!(observed.get(), Some(bytes.len() as u64));
            });
        }
    }

    // AR-SER-031 compositional parts A and C: this is a schema-shaped,
    // canonical Program payload whose four Bytes literals are synthesized on
    // demand. Its complete logical file is exactly 2^30 bytes.
    #[test]
    fn maximum_file_size_is_accepted_without_materializing_one_gibibyte() {
        let all_maximum = literal_program(5, &[MAX_ITEM_SIZE; 4]);
        let oversized_len = ENVELOPE_LEN as u64 + all_maximum.logical_len();
        let excess = oversized_len - MAX_FILE_SIZE as u64;
        assert!(excess < MAX_ITEM_SIZE);
        let final_length = MAX_ITEM_SIZE - excess;

        let exact = literal_program(
            5,
            &[MAX_ITEM_SIZE, MAX_ITEM_SIZE, MAX_ITEM_SIZE, final_length],
        );
        let exact_file_len = ENVELOPE_LEN as u64 + exact.logical_len();
        assert_eq!(exact_file_len, MAX_FILE_SIZE as u64);
        assert_eq!(validate_file_size(exact_file_len), Ok(()));
        assert_eq!(validate_canonical_input(&exact), Ok(()));

        let too_large = literal_program(
            5,
            &[
                MAX_ITEM_SIZE,
                MAX_ITEM_SIZE,
                MAX_ITEM_SIZE,
                final_length + 1,
            ],
        );
        let too_large_len = ENVELOPE_LEN as u64 + too_large.logical_len();
        assert_eq!(too_large_len, MAX_FILE_SIZE as u64 + 1);
        assert_eq!(
            validate_file_size(too_large_len),
            Err(DecodeError::ResourceLimitExceeded)
        );
    }

    // AR-SER-032: exact 2^28 Text and Bytes payloads traverse the production
    // canonical scanner, including streaming UTF-8 validation for Text.
    #[test]
    fn maximum_text_and_bytes_items_are_accepted_virtually() {
        for kind_tag in [4, 5] {
            let exact = literal_program(kind_tag, &[MAX_ITEM_SIZE]);
            assert_eq!(validate_canonical_input(&exact), Ok(()));

            let too_large = literal_program(kind_tag, &[MAX_ITEM_SIZE + 1]);
            assert_eq!(
                validate_canonical_input(&too_large),
                Err(DecodeError::ResourceLimitExceeded)
            );
        }
    }

    // AR-SER-032: the exact 2^24 Call-argument collection is logically
    // iterated by the production scanner without retaining decoded elements.
    #[test]
    fn maximum_variable_collection_is_accepted_virtually() {
        let exact = call_arguments_program(MAX_COLLECTION_COUNT);
        assert_eq!(validate_canonical_input(&exact), Ok(()));

        let too_large = call_arguments_program(MAX_COLLECTION_COUNT + 1);
        assert_eq!(
            validate_canonical_input(&too_large),
            Err(DecodeError::ResourceLimitExceeded)
        );
    }

    // AR-SER-032 and MNIR-SER-082: depth is small enough for ordinary
    // concrete input. Top-level payload depth 16 succeeds; depth 17 fails.
    #[test]
    fn exact_nesting_depth_boundary_uses_concrete_bytes() {
        let mut depth_16 = vec![0x81; 15];
        depth_16.push(0x00);
        assert_eq!(validate_canonical_cbor(&depth_16), Ok(()));

        let mut depth_17 = vec![0x81; 16];
        depth_17.push(0x00);
        assert_eq!(
            validate_canonical_cbor(&depth_17),
            Err(DecodeError::ResourceLimitExceeded)
        );
    }

    // AR-SER-004: every schema tag family rejects unknown alternatives and
    // fixed composites reject the wrong arity/type.
    #[test]
    fn unknown_schema_tags_and_wrong_shapes_are_rejected() {
        assert_eq!(
            decode_intrinsic(&mut Decoder::new(&[0x06])),
            Err(DecodeError::InvalidWireSchema)
        );
        assert_eq!(
            decode_persistence_value_type(&mut Decoder::new(&[0x82, 0x02, 0x00])),
            Err(DecodeError::InvalidWireSchema)
        );
        assert_eq!(
            decode_expression_kind(&mut Decoder::new(&[0x81, 0x15])),
            Err(DecodeError::InvalidWireSchema)
        );
        assert_eq!(
            decode_terminator(&mut Decoder::new(&[0x81, 0x02])),
            Err(DecodeError::InvalidWireSchema)
        );
        assert_eq!(
            decode_revision_counter(&mut Decoder::new(&[0x81, 0x02])),
            Err(DecodeError::InvalidWireSchema)
        );
        assert_eq!(
            decode_string_option(&mut Decoder::new(&[0x81, 0x02])),
            Err(DecodeError::InvalidWireSchema)
        );
        assert_eq!(
            fixed_array(&mut Decoder::new(&[0x81, 0x00]), 2),
            Err(DecodeError::InvalidWireSchema)
        );
        assert_eq!(
            fixed_array(&mut Decoder::new(&[0x00]), 2),
            Err(DecodeError::InvalidWireSchema)
        );
    }

    // AR-SER-007, AR-SER-009, AR-SER-011 through AR-SER-017, and
    // AR-SER-037: stable wire tags are independent of Rust enum order.
    #[test]
    fn every_type_expression_and_terminator_tag_is_exact() {
        for (intrinsic, tag) in [
            (IntrinsicType::Int32, 0),
            (IntrinsicType::Int64, 1),
            (IntrinsicType::Bool, 2),
            (IntrinsicType::Unit, 3),
            (IntrinsicType::Text, 4),
            (IntrinsicType::Bytes, 5),
        ] {
            assert_eq!(intrinsic_tag(intrinsic), tag);
        }

        let mut program = MnirProgram::new().unwrap();
        let mut transaction = program.begin_transaction();
        let module = transaction.add_module().unwrap();
        let domain = transaction
            .add_domain_type(module, IntrinsicType::Int32)
            .unwrap();
        let function = transaction
            .add_function(module, IntrinsicType::Unit)
            .unwrap();
        let parameter = transaction
            .add_parameter(function, IntrinsicType::Unit)
            .unwrap();
        let block = transaction.create_function_body(function).unwrap();
        let second_block = transaction.add_block(function).unwrap();
        let expression = transaction.add_unit_literal(block).unwrap();
        let kinds = [
            ExpressionKind::Int32Literal(0),
            ExpressionKind::Int64Literal(0),
            ExpressionKind::BoolLiteral(false),
            ExpressionKind::UnitLiteral,
            ExpressionKind::TextLiteral(String::new()),
            ExpressionKind::BytesLiteral(Vec::new()),
            ExpressionKind::ParameterReference(parameter),
            ExpressionKind::Add {
                left: expression,
                right: expression,
            },
            ExpressionKind::Subtract {
                left: expression,
                right: expression,
            },
            ExpressionKind::Multiply {
                left: expression,
                right: expression,
            },
            ExpressionKind::Divide {
                left: expression,
                right: expression,
            },
            ExpressionKind::Remainder {
                left: expression,
                right: expression,
            },
            ExpressionKind::Equal {
                left: expression,
                right: expression,
            },
            ExpressionKind::NotEqual {
                left: expression,
                right: expression,
            },
            ExpressionKind::LessThan {
                left: expression,
                right: expression,
            },
            ExpressionKind::LessThanOrEqual {
                left: expression,
                right: expression,
            },
            ExpressionKind::GreaterThan {
                left: expression,
                right: expression,
            },
            ExpressionKind::GreaterThanOrEqual {
                left: expression,
                right: expression,
            },
            ExpressionKind::Call {
                target: function,
                arguments: vec![expression],
            },
            ExpressionKind::DomainConstruct {
                type_id: domain,
                value: expression,
            },
            ExpressionKind::DomainProject { value: expression },
        ];
        for (expected_tag, kind) in kinds.into_iter().enumerate() {
            let mut encoder = Encoder::new(LimitedVec::default());
            encode_expression_kind(&mut encoder, &kind).unwrap();
            let bytes = encoder.into_writer().bytes;
            let mut decoder = Decoder::new(&bytes);
            decoder.array().unwrap().unwrap();
            assert_eq!(decoder.u64().unwrap(), expected_tag as u64);
        }

        for (terminator, expected_tag) in [
            (Terminator::Return { expression }, 0),
            (
                Terminator::Branch {
                    condition: expression,
                    true_block: block,
                    false_block: second_block,
                },
                1,
            ),
        ] {
            let mut encoder = Encoder::new(LimitedVec::default());
            encode_terminator(&mut encoder, terminator).unwrap();
            let bytes = encoder.into_writer().bytes;
            let mut decoder = Decoder::new(&bytes);
            decoder.array().unwrap().unwrap();
            assert_eq!(decoder.u64().unwrap(), expected_tag);
        }
    }
}
