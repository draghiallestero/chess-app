use backend::BitBoard;
use backend::Board;
use bit_iter::BitIter;
use iced::Background;
use iced::Theme;
use iced::color;
use iced::theme;
use iced::theme::palette::Pair;
use iced::widget::container::Style;
use iced::widget::{
    Column, Container, Grid, button, center, column, container, grid, text, themer,
};

#[derive(Default)]
struct Counter {
    value: i32,
    board: Board,
}

#[derive(Clone)]
pub enum Message {
    Increment,
    Decrement,
}

impl Counter {
    pub fn view(&self) -> Grid<Message> {
        // Create the style for the board squares
        let board_square_style_factory = |pos| {
            move |theme: &Theme| {
                let off_white = Background::Color(color!(238, 238, 210)).into();
                let green = Background::Color(color!(118, 150, 86)).into();

                let (x, y) = (pos / 8, pos % 8);
                let background_color = { if (x + y) % 2 == 0 { off_white } else { green } };

                container::Style {
                    background: background_color,
                    ..Style::default()
                }
            }
        };

        // Create the board squares
        let board_square_container_factory = |pos| {
            // Figure out what to display
            let mut square_text = String::default();
            let mut process_piece = |piece: BitBoard, piece_str| {
                let result = BitIter::from(piece.board).find(move |x| *x == pos);
                match result {
                    Some(_) => square_text = String::from(piece_str),
                    None => (),
                };
            };
            process_piece(self.board.player.pawns, "♙");
            process_piece(self.board.player.rooks, "♖");
            process_piece(self.board.player.knights, "♘");
            process_piece(self.board.player.bishops, "♗");
            process_piece(self.board.player.queens, "♕");
            process_piece(self.board.player.kings, "♔");
            process_piece(self.board.opponent.pawns, "♟");
            process_piece(self.board.opponent.rooks, "♜");
            process_piece(self.board.opponent.knights, "♞");
            process_piece(self.board.opponent.bishops, "♝");
            process_piece(self.board.opponent.queens, "♛");
            process_piece(self.board.opponent.kings, "♚");
            center(text(square_text).size(100)).style(board_square_style_factory(pos))
        };
        grid!(
            board_square_container_factory(56),
            board_square_container_factory(57),
            board_square_container_factory(58),
            board_square_container_factory(59),
            board_square_container_factory(60),
            board_square_container_factory(61),
            board_square_container_factory(62),
            board_square_container_factory(63),
            board_square_container_factory(48),
            board_square_container_factory(49),
            board_square_container_factory(50),
            board_square_container_factory(51),
            board_square_container_factory(52),
            board_square_container_factory(53),
            board_square_container_factory(54),
            board_square_container_factory(55),
            board_square_container_factory(40),
            board_square_container_factory(41),
            board_square_container_factory(42),
            board_square_container_factory(43),
            board_square_container_factory(44),
            board_square_container_factory(45),
            board_square_container_factory(46),
            board_square_container_factory(47),
            board_square_container_factory(32),
            board_square_container_factory(33),
            board_square_container_factory(34),
            board_square_container_factory(35),
            board_square_container_factory(36),
            board_square_container_factory(37),
            board_square_container_factory(38),
            board_square_container_factory(39),
            board_square_container_factory(24),
            board_square_container_factory(25),
            board_square_container_factory(26),
            board_square_container_factory(27),
            board_square_container_factory(28),
            board_square_container_factory(29),
            board_square_container_factory(30),
            board_square_container_factory(31),
            board_square_container_factory(16),
            board_square_container_factory(17),
            board_square_container_factory(18),
            board_square_container_factory(19),
            board_square_container_factory(20),
            board_square_container_factory(21),
            board_square_container_factory(22),
            board_square_container_factory(23),
            board_square_container_factory(08),
            board_square_container_factory(09),
            board_square_container_factory(10),
            board_square_container_factory(11),
            board_square_container_factory(12),
            board_square_container_factory(13),
            board_square_container_factory(14),
            board_square_container_factory(15),
            board_square_container_factory(00),
            board_square_container_factory(01),
            board_square_container_factory(02),
            board_square_container_factory(03),
            board_square_container_factory(04),
            board_square_container_factory(05),
            board_square_container_factory(06),
            board_square_container_factory(07),
        )
        .columns(8)
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Increment => {
                self.value += 1;
            }
            Message::Decrement => {
                self.value -= 1;
            }
        }
    }
}

fn main() -> iced::Result {
    // iced::run(Counter::update, Counter::view)
    iced::application(Counter::default, Counter::update, Counter::view)
        // .theme(Theme::TokyoNight)
        .title("Alessandro's Chess Application")
        // .resizable(false)
        .run()
}

#[test]
fn it_counts_properly() {
    let mut counter = Counter::default();

    counter.update(Message::Increment);
    counter.update(Message::Increment);
    counter.update(Message::Decrement);

    assert_eq!(counter.value, 1);
}
