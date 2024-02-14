// Based on https://chat.openai.com/c/049ae523-7ee0-47a8-b440-cca5b1ae7cd6
// original question:
// I want my_request to be serialized into a byte array s[u8;2] with
// the first byte as the frequency and the second byte, bits 3 to 4 as the FreqUnit.
// How would I do this?

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum FreqUnit {
    Kilohertz = 0,
    Megahertz = 1,
    Gigahertz = 2,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MyRequest {
    freq: u8,
    unit: FreqUnit,
}

impl From<MyRequest> for [u8; 2] {
    fn from(request: MyRequest) -> Self {
        let freq_byte = request.freq;
        let unit_byte = match request.unit {
            FreqUnit::Kilohertz => 0b0000_0000,
            FreqUnit::Megahertz => 0b0000_1000,
            FreqUnit::Gigahertz => 0b0001_1000,
        };

        [freq_byte, unit_byte]
    }
}

impl TryFrom<[u8; 2]> for MyRequest {
    type Error = DeserializeError;

    fn try_from(bytes: [u8; 2]) -> Result<Self, Self::Error> {
        let freq = bytes[0];
        let unit_byte = bytes[1] & 0b0001_1000;

        let unit = match unit_byte {
            0b0000_0000 => FreqUnit::Kilohertz,
            0b0000_1000 => FreqUnit::Megahertz,
            0b0001_1000 => FreqUnit::Gigahertz,
            _ => return Err(DeserializeError::Enumeration),
        };

        Ok(MyRequest { freq, unit })
    }
}

#[derive(Debug)]
pub enum DeserializeError {
    Enumeration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let request = MyRequest {
            freq: 42,
            unit: FreqUnit::Megahertz,
        };

        let buf: [u8; 2] = request.into();

        assert_eq!(buf, [42, 0b0000_1000]);
    }

    #[test]
    fn deserialize() {
        let expected = MyRequest {
            freq: 12,
            unit: FreqUnit::Gigahertz,
        };

        let buf: [u8; 2] = [12, 0b0001_1000];

        let request = MyRequest::try_from(buf);

        assert!(request.is_ok());

        assert_eq!(request.unwrap(), expected);
    }
}
