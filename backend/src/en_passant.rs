#[derive(Clone, Copy)]
pub struct EnPassant {
    pub target_pos: u8,
    pub attacker_pos_list: [u8; 2],
}

impl Default for EnPassant {
    fn default() -> Self {
        EnPassant {
            target_pos: 64,
            attacker_pos_list: [64; 2],
        }
    }
}

impl EnPassant {
    pub fn flip_view(&self) -> EnPassant {
        let flip = |x| {
            if x == 64 { x } else { 63 - x }
        };
        EnPassant {
            target_pos: flip(self.target_pos),
            attacker_pos_list: [
                flip(self.attacker_pos_list[0]),
                flip(self.attacker_pos_list[1]),
            ],
        }
    }
}
