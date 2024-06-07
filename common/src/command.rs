pub trait Command {
    fn opcode(&self) -> u8;
}
