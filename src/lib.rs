mod position;

#[cfg(test)]
mod tests {
    use crate::position::{Piece, Position, Side};
    #[test]
    fn new_position() {
        let op = "Ra1 Db1 Rc1 Rd1 De1 Rf1 Cg1 Rh1 Ra2 Hb2 Cc2 Ed2 Me2 Rf2 Hg2 Rh2
        ha7 mb7 cc7 dd7 ee7 cf7 hg7 rh7 ra8 rb8 rc8 rd8 de8 rf8 rg8 rh8";
        let pos = Position::from_opening_str(op);
        assert!(pos.is_some());
    }
}
