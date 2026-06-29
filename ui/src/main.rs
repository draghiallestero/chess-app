use backend::BitBoard;
use backend::Board;
use bit_iter::BitIter;
use iced::Background;
use iced::Point;
use iced::Theme;
use iced::color;
use iced::theme;
use iced::widget::container::Style;
use iced::widget::float;
use iced::widget::mouse_area;
use iced::widget::{
    Column, Container, Grid, Stack, button, center, column, container, grid, stack, text, themer,
};

#[derive(Default)]
struct BoardWidget {
    board: Board,
    dragged_piece: Option<(usize, String)>,
}

#[derive(Clone)]
pub enum Message {
    HoldDraggedPiece((usize, String)),
    ReleaseDraggedPiece,
    MouseMoved(Point),
}

impl BoardWidget {
    pub fn view(&self) -> Stack<Message> {
        let text_size = 100;

        // Create the style for the board squares
        let board_square_style_factory = |pos| {
            move |_: &Theme| {
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
        let board_square_widget_factory = |pos| {
            // Figure out what to display
            let mut square_text = String::default();
            let mut process_piece = |piece: BitBoard, piece_str| {
                let result = BitIter::from(piece.board).find(move |x| *x == pos);
                match result {
                    Some(_) => {
                        // Pieces being dragged are empty
                        match &self.dragged_piece {
                            Some((dragged_pos, _)) => {
                                if pos == *dragged_pos {
                                    return;
                                }
                            }
                            None => (),
                        }
                        square_text = String::from(piece_str);
                    }
                    None => return,
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
            let text_widget = center(text(square_text.clone()).size(text_size))
                .style(board_square_style_factory(pos));

            // Create the mouse that will inform us of clicks
            mouse_area(text_widget)
                .on_press(Message::HoldDraggedPiece((pos, square_text)))
                .on_release(Message::ReleaseDraggedPiece)
                .on_move(|point| Message::MouseMoved(point))
        };
        // let board_square_widgets = [
        //     board_square_widget_factory(56),
        //     board_square_widget_factory(57),
        //     board_square_widget_factory(58),
        //     board_square_widget_factory(59),
        //     board_square_widget_factory(60),
        //     board_square_widget_factory(61),
        //     board_square_widget_factory(62),
        //     board_square_widget_factory(63),
        //     board_square_widget_factory(48),
        //     board_square_widget_factory(49),
        //     board_square_widget_factory(50),
        //     board_square_widget_factory(51),
        //     board_square_widget_factory(52),
        //     board_square_widget_factory(53),
        //     board_square_widget_factory(54),
        //     board_square_widget_factory(55),
        //     board_square_widget_factory(40),
        //     board_square_widget_factory(41),
        //     board_square_widget_factory(42),
        //     board_square_widget_factory(43),
        //     board_square_widget_factory(44),
        //     board_square_widget_factory(45),
        //     board_square_widget_factory(46),
        //     board_square_widget_factory(47),
        //     board_square_widget_factory(32),
        //     board_square_widget_factory(33),
        //     board_square_widget_factory(34),
        //     board_square_widget_factory(35),
        //     board_square_widget_factory(36),
        //     board_square_widget_factory(37),
        //     board_square_widget_factory(38),
        //     board_square_widget_factory(39),
        //     board_square_widget_factory(24),
        //     board_square_widget_factory(25),
        //     board_square_widget_factory(26),
        //     board_square_widget_factory(27),
        //     board_square_widget_factory(28),
        //     board_square_widget_factory(29),
        //     board_square_widget_factory(30),
        //     board_square_widget_factory(31),
        //     board_square_widget_factory(16),
        //     board_square_widget_factory(17),
        //     board_square_widget_factory(18),
        //     board_square_widget_factory(19),
        //     board_square_widget_factory(20),
        //     board_square_widget_factory(21),
        //     board_square_widget_factory(22),
        //     board_square_widget_factory(23),
        //     board_square_widget_factory(08),
        //     board_square_widget_factory(09),
        //     board_square_widget_factory(10),
        //     board_square_widget_factory(11),
        //     board_square_widget_factory(12),
        //     board_square_widget_factory(13),
        //     board_square_widget_factory(14),
        //     board_square_widget_factory(15),
        //     board_square_widget_factory(00),
        //     board_square_widget_factory(01),
        //     board_square_widget_factory(02),
        //     board_square_widget_factory(03),
        //     board_square_widget_factory(04),
        //     board_square_widget_factory(05),
        //     board_square_widget_factory(06),
        //     board_square_widget_factory(07),
        // ];

        // Create the floating piece
        let floating_piece = float(center(
            text(match &self.dragged_piece {
                Some((_, piece_str)) => piece_str.clone(),
                None => String::default(),
            })
            .size(text_size),
        ));

        // Create the board widget
        stack![
            grid!(
                board_square_widget_factory(56),
                board_square_widget_factory(57),
                board_square_widget_factory(58),
                board_square_widget_factory(59),
                board_square_widget_factory(60),
                board_square_widget_factory(61),
                board_square_widget_factory(62),
                board_square_widget_factory(63),
                board_square_widget_factory(48),
                board_square_widget_factory(49),
                board_square_widget_factory(50),
                board_square_widget_factory(51),
                board_square_widget_factory(52),
                board_square_widget_factory(53),
                board_square_widget_factory(54),
                board_square_widget_factory(55),
                board_square_widget_factory(40),
                board_square_widget_factory(41),
                board_square_widget_factory(42),
                board_square_widget_factory(43),
                board_square_widget_factory(44),
                board_square_widget_factory(45),
                board_square_widget_factory(46),
                board_square_widget_factory(47),
                board_square_widget_factory(32),
                board_square_widget_factory(33),
                board_square_widget_factory(34),
                board_square_widget_factory(35),
                board_square_widget_factory(36),
                board_square_widget_factory(37),
                board_square_widget_factory(38),
                board_square_widget_factory(39),
                board_square_widget_factory(24),
                board_square_widget_factory(25),
                board_square_widget_factory(26),
                board_square_widget_factory(27),
                board_square_widget_factory(28),
                board_square_widget_factory(29),
                board_square_widget_factory(30),
                board_square_widget_factory(31),
                board_square_widget_factory(16),
                board_square_widget_factory(17),
                board_square_widget_factory(18),
                board_square_widget_factory(19),
                board_square_widget_factory(20),
                board_square_widget_factory(21),
                board_square_widget_factory(22),
                board_square_widget_factory(23),
                board_square_widget_factory(08),
                board_square_widget_factory(09),
                board_square_widget_factory(10),
                board_square_widget_factory(11),
                board_square_widget_factory(12),
                board_square_widget_factory(13),
                board_square_widget_factory(14),
                board_square_widget_factory(15),
                board_square_widget_factory(00),
                board_square_widget_factory(01),
                board_square_widget_factory(02),
                board_square_widget_factory(03),
                board_square_widget_factory(04),
                board_square_widget_factory(05),
                board_square_widget_factory(06),
                board_square_widget_factory(07),
            )
            .columns(8),
            floating_piece
        ]
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::HoldDraggedPiece(dragged_piece) => self.dragged_piece = dragged_piece.into(),
            Message::ReleaseDraggedPiece =>
            // Some complicated stuff about making a move, idk
            {
                self.dragged_piece = None
            }
            Message::MouseMoved(point) => {
                println!("{}", point);
                ()
            }
        }
    }
}

fn main() -> iced::Result {
    // iced::run(BoardWidget::update, BoardWidget::view)
    iced::application(BoardWidget::default, BoardWidget::update, BoardWidget::view)
        // .theme(Theme::TokyoNight)
        .title("Alessandro's Chess Application")
        // .resizable(false)
        .run()
}
