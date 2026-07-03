use std::alloc::dealloc;
use std::mem::swap;

use backend::BitBoard;
use backend::Board;
use backend::Move;
use bit_iter::BitIter;
use iced::Background;
use iced::Border;
use iced::Event;
use iced::Point;
use iced::Program;
use iced::Size;
use iced::Subscription;
use iced::Theme;
use iced::Vector;
use iced::color;
use iced::debug;
use iced::event;
use iced::mouse;
use iced::theme;
use iced::widget::MouseArea;
use iced::widget::Text;
use iced::widget::container::Style;
use iced::widget::float;
use iced::widget::mouse_area;
use iced::widget::pane_grid::default;
use iced::widget::pin;
use iced::widget::row;
use iced::widget::{
    Column, Container, Grid, Stack, button, center, column, container, grid, stack, text, themer,
};
use iced::window;
use iced::window::Position;
use iced::window::Settings;

struct BoardWidget {
    board: Board,
    held_piece_pos: Option<u8>,
    mouse_pos: Point,
    move_count: i32,
    legal_moves: Vec<Move>,
}

impl Default for BoardWidget {
    fn default() -> Self {
        let board = Board::default();
        let legal_moves = board.generate_legal_moves();
        BoardWidget {
            board: board,
            held_piece_pos: None,
            mouse_pos: Point::default(),
            move_count: 0,
            legal_moves: legal_moves,
        }
    }
}

#[derive(Clone)]
pub enum Message {
    HoldPiece(u8),
    ReleasePiece(u8),
    MouseMoved(Point),
}

#[derive(Clone)]
pub struct PieceTextMap {
    pawns: &'static str,
    rooks: &'static str,
    knights: &'static str,
    bishops: &'static str,
    queens: &'static str,
    kings: &'static str,
}

impl PieceTextMap {
    pub fn new(move_count: i32) -> PieceTextMap {
        if move_count % 2 == 0 {
            PieceTextMap {
                pawns: "♙",
                rooks: "♖",
                knights: "♘",
                bishops: "♗",
                queens: "♔",
                kings: "♕",
            }
        } else {
            PieceTextMap {
                pawns: "♟",
                rooks: "♜",
                knights: "♞",
                bishops: "♝",
                queens: "♚",
                kings: "♛",
            }
        }
    }
}

impl BoardWidget {
    pub fn view(&'_ self) -> MouseArea<'_, Message> {
        let text_size = 100;

        // Determine player and opponent text maps
        let player_piece_text_map = PieceTextMap::new(self.move_count);
        let opponent_piece_text_map = PieceTextMap::new(self.move_count + 1);

        // Which squares the held piece can be moved to
        let legal_move_targets = match self.held_piece_pos {
            Some(from) => self
                .legal_moves
                .iter()
                .filter(|_move| _move.from == from)
                .map(|_move| _move.to as u8)
                .collect(),
            None => Vec::default(),
        };

        // Figure out text what to print on the board
        let mut board_square_text: [&'static str; 64] = std::array::from_fn(|pos| {
            // Process pieces
            let mut square_text: Option<&'static str> = None;
            let mut process_piece = |piece: BitBoard, piece_text| {
                // Do nothing if text was found
                if !square_text.is_some() && piece.get(pos as u8) {
                    square_text = Some(piece_text);
                }
            };
            process_piece(self.board.player.pawns, player_piece_text_map.pawns);
            process_piece(self.board.player.rooks, player_piece_text_map.rooks);
            process_piece(self.board.player.knights, player_piece_text_map.knights);
            process_piece(self.board.player.bishops, player_piece_text_map.bishops);
            process_piece(self.board.player.queens, player_piece_text_map.queens);
            process_piece(self.board.player.kings, player_piece_text_map.kings);
            process_piece(self.board.opponent.pawns, opponent_piece_text_map.pawns);
            process_piece(self.board.opponent.rooks, opponent_piece_text_map.rooks);
            process_piece(self.board.opponent.knights, opponent_piece_text_map.knights);
            process_piece(self.board.opponent.bishops, opponent_piece_text_map.bishops);
            process_piece(self.board.opponent.queens, opponent_piece_text_map.queens);
            process_piece(self.board.opponent.kings, opponent_piece_text_map.kings);

            square_text.unwrap_or("")
        });

        // Handle floating square
        let floating_square = float(center(
            text(match self.held_piece_pos {
                Some(pos) => {
                    let floating_square_text = board_square_text[pos as usize];
                    board_square_text[pos as usize] = "";
                    floating_square_text
                }
                None => "",
            })
            .size(text_size),
        ))
        .translate(move |r1, r2| Vector {
            x: self.mouse_pos.x - r1.width / 2.0,
            y: self.mouse_pos.y - r1.height / 2.0,
        });

        // Create the board squares
        let board_square_widget_factory = |pos| {
            // Figure out what to display
            let pos = pos as u8;

            // Create the style for the board squares
            let style_fn_factory = || {
                let (x, y) = (pos / 8, pos % 8);

                let off_white = Background::Color(color!(238, 238, 210)).into();
                let green = Background::Color(color!(118, 150, 86)).into();
                let background_color = { if (x + y) % 2 == 0 { off_white } else { green } };

                // let border_color = color!(247, 212, 102);
                let border_color = color!(82, 170, 243);
                // let border_color = color!(235, 110, 78);
                let border_width = if legal_move_targets.iter().any(|x| *x == pos) {
                    10.0
                } else {
                    0.0
                };
                let border = Border {
                    color: border_color,
                    width: border_width,
                    ..Border::default()
                };

                move |_: &Theme| container::Style {
                    background: background_color,
                    border: border,
                    ..Style::default()
                }
            };

            let text_widget = center(text(board_square_text[pos as usize]).size(text_size))
                .style(style_fn_factory());

            // Create the mouse area that will inform us of clicks
            mouse_area(text_widget)
                .on_press(Message::HoldPiece(pos))
                .on_release(Message::ReleasePiece(pos))
        };

        // Create the mouse area of the overall application
        let mouse_area_widget_applicator =
            |widget| mouse_area(widget).on_move(|pos| Message::MouseMoved(pos));

        // Create the board widget
        mouse_area_widget_applicator(stack![
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
            floating_square
        ])
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::HoldPiece(held_piece) => {
                println!("HoldPiece");
                self.held_piece_pos = held_piece.into()
            }
            Message::ReleasePiece(target) => {
                println!("ReleasePiece");
                match self
                    .legal_moves
                    .iter()
                    .find(|_move| _move.from == self.held_piece_pos.unwrap() && _move.to == target)
                {
                    Some(_move) => {
                        self.board = self.board.apply_move(_move).flip_view();
                        self.legal_moves = self.board.generate_legal_moves();
                        self.move_count += 1;
                    }
                    None => (),
                };
                self.held_piece_pos = None;
            }
            Message::MouseMoved(pos) => {
                println!("{}", pos);
                self.mouse_pos = pos;
            }
        }
    }
}

fn main() -> iced::Result {
    // iced::run(BoardWidget::update, BoardWidget::view)
    iced::application(BoardWidget::default, BoardWidget::update, BoardWidget::view)
        // .theme(Theme::TokyoNight)
        .title("Alessandro's Chess Application")
        // .subscription(BoardWidget::subscription)
        // .resizable(false)
        .window_size(Size {
            width: 1000.0,
            height: 1000.0,
        })
        .run()
}
