use crate::domain::error::ProtocolError;

// [ 4 bytes length ][ 1 byte message type ][ payload: serialized.len() ]

pub trait Package
where Self: TryFrom<Vec<u8>> + Into<Vec<u8>> + Clone + Sized {
    fn message_type(&self) -> u8;
    fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let serialized: Vec<u8> = self.clone().into();

        let body_len: usize = 1 + serialized.len();
        let len: u32 = u32::try_from(body_len).map_err(|_| ProtocolError::PacketTooLarge)?;

        let mut bytes: Vec<u8> = Vec::with_capacity(4 + body_len);
        bytes.extend_from_slice(&len.to_be_bytes());
        bytes.push(self.message_type());
        bytes.extend(&serialized);
        Ok(bytes)
    }
}
