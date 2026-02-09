use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use super::{CompatError, CompatResult};
use super::gateway::BinaryTernaryGateway;
use super::gateway::GatewayMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFormat {
    Binary,
    TernaryA,
    TernaryB,
    TernaryC,
    Mixed,
}

#[derive(Debug, Clone)]
pub struct DataBuffer {
    data: Vec<u8>,
    format: DataFormat,
    length: usize,
}

impl DataBuffer {
    pub fn new(format: DataFormat) -> Self {
        Self {
            data: Vec::new(),
            format,
            length: 0,
        }
    }

    pub fn from_binary(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
            format: DataFormat::Binary,
            length: data.len(),
        }
    }

    pub fn from_ternary_a(trits: &[i8]) -> Self {
        let data: Vec<u8> = trits.iter().map(|&t| t as u8).collect();
        Self {
            data,
            format: DataFormat::TernaryA,
            length: trits.len(),
        }
    }

    pub fn as_binary(&self) -> CompatResult<&[u8]> {
        if self.format != DataFormat::Binary {
            return Err(CompatError::ConversionFailed(
                String::from("Buffer is not in binary format"),
            ));
        }
        Ok(&self.data)
    }

    pub fn as_ternary_a(&self) -> CompatResult<Vec<i8>> {
        if self.format != DataFormat::TernaryA {
            return Err(CompatError::ConversionFailed(
                String::from("Buffer is not in TernaryA format"),
            ));
        }
        Ok(self.data.iter().map(|&b| b as i8).collect())
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn format(&self) -> &DataFormat {
        &self.format
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionStep {
    BinaryToTernaryA,
    TernaryAToBinary,
    TernaryAToB,
    TernaryBToA,
    TernaryAToC,
    TernaryCToA,
    TernaryBToC,
    TernaryCToB,
}

#[derive(Debug, Clone)]
pub struct ConversionPipeline {
    steps: Vec<ConversionStep>,
}

impl ConversionPipeline {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn add_step(&mut self, step: ConversionStep) {
        self.steps.push(step);
    }

    pub fn execute(&self, input: DataBuffer) -> CompatResult<DataBuffer> {
        let mut current = input;
        for &step in &self.steps {
            current = apply_conversion_step(step, current)?;
        }
        Ok(current)
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }
}

fn apply_conversion_step(step: ConversionStep, input: DataBuffer) -> CompatResult<DataBuffer> {
    match step {
        ConversionStep::BinaryToTernaryA => {
            let binary_data = input.as_binary()?;
            let trits = super::gateway::binary_bytes_to_ternary(binary_data);
            Ok(DataBuffer::from_ternary_a(&trits))
        }
        ConversionStep::TernaryAToBinary => {
            let trits = input.as_ternary_a()?;
            let bytes = super::gateway::ternary_to_binary_bytes(&trits)?;
            Ok(DataBuffer::from_binary(&bytes))
        }
        ConversionStep::TernaryAToB => {
            let trits = input.as_ternary_a()?;
            let rep_b: Vec<u8> = trits.iter().map(|&t| (t + 1) as u8).collect();
            let len = rep_b.len();
            Ok(DataBuffer {
                data: rep_b,
                format: DataFormat::TernaryB,
                length: len,
            })
        }
        ConversionStep::TernaryBToA => {
            if *input.format() != DataFormat::TernaryB {
                return Err(CompatError::ConversionFailed(
                    String::from("Expected TernaryB format"),
                ));
            }
            let trits: Vec<i8> = input.data.iter().map(|&b| b as i8 - 1).collect();
            Ok(DataBuffer::from_ternary_a(&trits))
        }
        ConversionStep::TernaryAToC => {
            let trits = input.as_ternary_a()?;
            let rep_c: Vec<u8> = trits.iter().map(|&t| (t + 2) as u8).collect();
            let len = rep_c.len();
            Ok(DataBuffer {
                data: rep_c,
                format: DataFormat::TernaryC,
                length: len,
            })
        }
        ConversionStep::TernaryCToA => {
            if *input.format() != DataFormat::TernaryC {
                return Err(CompatError::ConversionFailed(
                    String::from("Expected TernaryC format"),
                ));
            }
            let trits: Vec<i8> = input.data.iter().map(|&b| b as i8 - 2).collect();
            Ok(DataBuffer::from_ternary_a(&trits))
        }
        ConversionStep::TernaryBToC => {
            if *input.format() != DataFormat::TernaryB {
                return Err(CompatError::ConversionFailed(
                    String::from("Expected TernaryB format"),
                ));
            }
            let rep_c: Vec<u8> = input.data.iter().map(|&b| b + 1).collect();
            let len = rep_c.len();
            Ok(DataBuffer {
                data: rep_c,
                format: DataFormat::TernaryC,
                length: len,
            })
        }
        ConversionStep::TernaryCToB => {
            if *input.format() != DataFormat::TernaryC {
                return Err(CompatError::ConversionFailed(
                    String::from("Expected TernaryC format"),
                ));
            }
            let rep_b: Vec<u8> = input.data.iter().map(|&b| b - 1).collect();
            let len = rep_b.len();
            Ok(DataBuffer {
                data: rep_b,
                format: DataFormat::TernaryB,
                length: len,
            })
        }
    }
}

pub struct UniversalAdapter {
    gateway: BinaryTernaryGateway,
    pipelines: BTreeMap<String, ConversionPipeline>,
    conversions: u64,
}

impl UniversalAdapter {
    pub fn new() -> Self {
        Self {
            gateway: BinaryTernaryGateway::new(GatewayMode::Balanced),
            pipelines: BTreeMap::new(),
            conversions: 0,
        }
    }

    pub fn convert(&mut self, input: DataBuffer, target: DataFormat) -> CompatResult<DataBuffer> {
        self.conversions += 1;
        let current_format = *input.format();

        if current_format == target {
            return Ok(input);
        }

        match (current_format, target) {
            (DataFormat::Binary, DataFormat::TernaryA) => {
                let binary_data = input.as_binary()?;
                let trits = self.gateway.convert_to_ternary(binary_data)?;
                Ok(DataBuffer::from_ternary_a(&trits))
            }
            (DataFormat::TernaryA, DataFormat::Binary) => {
                let trits = input.as_ternary_a()?;
                let bytes = self.gateway.convert_to_binary(&trits)?;
                Ok(DataBuffer::from_binary(&bytes))
            }
            (DataFormat::Binary, DataFormat::TernaryB) => {
                let binary_data = input.as_binary()?;
                let trits = self.gateway.convert_to_ternary(binary_data)?;
                let buf_a = DataBuffer::from_ternary_a(&trits);
                apply_conversion_step(ConversionStep::TernaryAToB, buf_a)
            }
            (DataFormat::Binary, DataFormat::TernaryC) => {
                let binary_data = input.as_binary()?;
                let trits = self.gateway.convert_to_ternary(binary_data)?;
                let buf_a = DataBuffer::from_ternary_a(&trits);
                apply_conversion_step(ConversionStep::TernaryAToC, buf_a)
            }
            (DataFormat::TernaryA, DataFormat::TernaryB) => {
                apply_conversion_step(ConversionStep::TernaryAToB, input)
            }
            (DataFormat::TernaryA, DataFormat::TernaryC) => {
                apply_conversion_step(ConversionStep::TernaryAToC, input)
            }
            (DataFormat::TernaryB, DataFormat::TernaryA) => {
                apply_conversion_step(ConversionStep::TernaryBToA, input)
            }
            (DataFormat::TernaryB, DataFormat::TernaryC) => {
                apply_conversion_step(ConversionStep::TernaryBToC, input)
            }
            (DataFormat::TernaryB, DataFormat::Binary) => {
                let buf_a = apply_conversion_step(ConversionStep::TernaryBToA, input)?;
                let trits = buf_a.as_ternary_a()?;
                let bytes = self.gateway.convert_to_binary(&trits)?;
                Ok(DataBuffer::from_binary(&bytes))
            }
            (DataFormat::TernaryC, DataFormat::TernaryA) => {
                apply_conversion_step(ConversionStep::TernaryCToA, input)
            }
            (DataFormat::TernaryC, DataFormat::TernaryB) => {
                apply_conversion_step(ConversionStep::TernaryCToB, input)
            }
            (DataFormat::TernaryC, DataFormat::Binary) => {
                let buf_a = apply_conversion_step(ConversionStep::TernaryCToA, input)?;
                let trits = buf_a.as_ternary_a()?;
                let bytes = self.gateway.convert_to_binary(&trits)?;
                Ok(DataBuffer::from_binary(&bytes))
            }
            _ => Err(CompatError::UnsupportedType(
                String::from("Unsupported format conversion"),
            )),
        }
    }

    pub fn register_pipeline(&mut self, name: &str, pipeline: ConversionPipeline) {
        self.pipelines.insert(String::from(name), pipeline);
    }

    pub fn execute_pipeline(&mut self, name: &str, input: DataBuffer) -> CompatResult<DataBuffer> {
        let pipeline = self
            .pipelines
            .get(name)
            .ok_or_else(|| CompatError::AdapterError(String::from("Pipeline not found")))?
            .clone();
        self.conversions += 1;
        pipeline.execute(input)
    }

    pub fn auto_detect_format(data: &[u8]) -> DataFormat {
        if data.is_empty() {
            return DataFormat::Binary;
        }

        let all_rep_b = data.iter().all(|&b| b <= 2);
        if all_rep_b {
            return DataFormat::TernaryB;
        }

        let all_signed_trits = data.iter().all(|&b| {
            let v = b as i8;
            v == -1 || v == 0 || v == 1
        });
        if all_signed_trits {
            return DataFormat::TernaryA;
        }

        DataFormat::Binary
    }

    pub fn conversion_count(&self) -> u64 {
        self.conversions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_buffer_new() {
        let buf = DataBuffer::new(DataFormat::Binary);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert_eq!(*buf.format(), DataFormat::Binary);
    }

    #[test]
    fn test_data_buffer_from_binary() {
        let buf = DataBuffer::from_binary(&[1, 2, 3]);
        assert_eq!(buf.len(), 3);
        assert!(!buf.is_empty());
        assert_eq!(*buf.format(), DataFormat::Binary);
        assert_eq!(buf.as_binary().unwrap(), &[1, 2, 3]);
    }

    #[test]
    fn test_data_buffer_from_ternary_a() {
        let trits = [0i8, 1, -1, 0, 1, -1];
        let buf = DataBuffer::from_ternary_a(&trits);
        assert_eq!(buf.len(), 6);
        assert_eq!(*buf.format(), DataFormat::TernaryA);
        let recovered = buf.as_ternary_a().unwrap();
        assert_eq!(recovered, trits);
    }

    #[test]
    fn test_data_buffer_wrong_format_access() {
        let buf = DataBuffer::from_binary(&[1, 2]);
        assert!(buf.as_ternary_a().is_err());

        let buf = DataBuffer::from_ternary_a(&[0, 1]);
        assert!(buf.as_binary().is_err());
    }

    #[test]
    fn test_binary_to_ternary_a_roundtrip() {
        let original = DataBuffer::from_binary(&[42, 0, 255]);
        let mut adapter = UniversalAdapter::new();
        let ternary = adapter.convert(original, DataFormat::TernaryA).unwrap();
        assert_eq!(*ternary.format(), DataFormat::TernaryA);
        let binary = adapter.convert(ternary, DataFormat::Binary).unwrap();
        assert_eq!(binary.as_binary().unwrap(), &[42, 0, 255]);
    }

    #[test]
    fn test_ternary_a_to_b_to_c_to_a_roundtrip() {
        let trits = [0i8, 1, -1, 0, 1, -1];
        let buf_a = DataBuffer::from_ternary_a(&trits);
        let mut adapter = UniversalAdapter::new();

        let buf_b = adapter.convert(buf_a, DataFormat::TernaryB).unwrap();
        assert_eq!(*buf_b.format(), DataFormat::TernaryB);

        let buf_c = adapter.convert(buf_b, DataFormat::TernaryC).unwrap();
        assert_eq!(*buf_c.format(), DataFormat::TernaryC);

        let buf_a2 = adapter.convert(buf_c, DataFormat::TernaryA).unwrap();
        assert_eq!(buf_a2.as_ternary_a().unwrap(), trits);
    }

    #[test]
    fn test_conversion_pipeline_multi_step() {
        let mut pipeline = ConversionPipeline::new();
        pipeline.add_step(ConversionStep::BinaryToTernaryA);
        pipeline.add_step(ConversionStep::TernaryAToB);
        assert_eq!(pipeline.step_count(), 2);

        let input = DataBuffer::from_binary(&[42]);
        let result = pipeline.execute(input).unwrap();
        assert_eq!(*result.format(), DataFormat::TernaryB);
    }

    #[test]
    fn test_conversion_pipeline_roundtrip() {
        let mut pipeline = ConversionPipeline::new();
        pipeline.add_step(ConversionStep::BinaryToTernaryA);
        pipeline.add_step(ConversionStep::TernaryAToBinary);

        let input = DataBuffer::from_binary(&[100]);
        let result = pipeline.execute(input).unwrap();
        assert_eq!(result.as_binary().unwrap(), &[100]);
    }

    #[test]
    fn test_universal_adapter_auto_detect_binary() {
        let format = UniversalAdapter::auto_detect_format(&[128, 200, 255]);
        assert_eq!(format, DataFormat::Binary);
    }

    #[test]
    fn test_universal_adapter_auto_detect_ternary_b() {
        let format = UniversalAdapter::auto_detect_format(&[0, 1, 2, 0, 1]);
        assert_eq!(format, DataFormat::TernaryB);
    }

    #[test]
    fn test_universal_adapter_auto_detect_empty() {
        let format = UniversalAdapter::auto_detect_format(&[]);
        assert_eq!(format, DataFormat::Binary);
    }

    #[test]
    fn test_universal_adapter_conversion_count() {
        let mut adapter = UniversalAdapter::new();
        let buf = DataBuffer::from_binary(&[1]);
        let _ = adapter.convert(buf, DataFormat::TernaryA).unwrap();
        assert_eq!(adapter.conversion_count(), 1);
    }

    #[test]
    fn test_pipeline_registration_and_execution() {
        let mut adapter = UniversalAdapter::new();
        let mut pipeline = ConversionPipeline::new();
        pipeline.add_step(ConversionStep::BinaryToTernaryA);
        adapter.register_pipeline("bin_to_a", pipeline);

        let input = DataBuffer::from_binary(&[10]);
        let result = adapter.execute_pipeline("bin_to_a", input).unwrap();
        assert_eq!(*result.format(), DataFormat::TernaryA);
    }

    #[test]
    fn test_pipeline_not_found() {
        let mut adapter = UniversalAdapter::new();
        let input = DataBuffer::from_binary(&[1]);
        assert!(adapter.execute_pipeline("nonexistent", input).is_err());
    }

    #[test]
    fn test_same_format_no_conversion() {
        let mut adapter = UniversalAdapter::new();
        let buf = DataBuffer::from_binary(&[5, 10]);
        let result = adapter.convert(buf, DataFormat::Binary).unwrap();
        assert_eq!(result.as_binary().unwrap(), &[5, 10]);
    }

    #[test]
    fn test_empty_buffer_conversion() {
        let mut adapter = UniversalAdapter::new();
        let buf = DataBuffer::from_binary(&[]);
        let result = adapter.convert(buf, DataFormat::TernaryA).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_single_byte_roundtrip_all_formats() {
        let mut adapter = UniversalAdapter::new();
        let original = DataBuffer::from_binary(&[73]);
        let a = adapter.convert(original, DataFormat::TernaryA).unwrap();
        let b = adapter.convert(a, DataFormat::TernaryB).unwrap();
        let c = adapter.convert(b, DataFormat::TernaryC).unwrap();
        let back_a = adapter.convert(c, DataFormat::TernaryA).unwrap();
        let back_bin = adapter.convert(back_a, DataFormat::Binary).unwrap();
        assert_eq!(back_bin.as_binary().unwrap(), &[73]);
    }

    #[test]
    fn test_ternary_b_to_binary() {
        let original = [42u8];
        let buf_bin = DataBuffer::from_binary(&original);
        let mut adapter = UniversalAdapter::new();

        let buf_a = adapter.convert(buf_bin, DataFormat::TernaryA).unwrap();
        let buf_b = adapter.convert(buf_a, DataFormat::TernaryB).unwrap();
        let buf_back = adapter.convert(buf_b, DataFormat::Binary).unwrap();

        assert_eq!(buf_back.as_binary().unwrap(), &original);
    }

    #[test]
    fn test_data_buffer_empty() {
        let buf = DataBuffer::new(DataFormat::TernaryA);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }
}
