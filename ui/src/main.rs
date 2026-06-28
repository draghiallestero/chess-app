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
}

#[derive(Clone)]
pub enum Message {
    Increment,
    Decrement,
}

impl Counter {
    pub fn view(&self) -> Grid<Message> {
        // // We use a column: a simple vertical layout
        // let c = column![
        //     // The increment button. We tell it to produce an
        //     // `Increment` message when pressed
        //     button("+").on_press(Message::Increment),
        //     // We show the value of the counter here
        //     text(self.value).size(50),
        //     // The decrement button. We tell it to produce a
        //     // `Decrement` message when pressed
        //     button("-").on_press(Message::Decrement),
        // ];

        // let default_style = Style::default();

        // let t = theme::default();

        let bb = container::bordered_box;

        // let themer = themer(Background(color!(118, 150, 86)), text(1));

        // Container::default();

        let mut pos = 0;

        let theme_fn = |theme: &Theme| {
            let palette = theme.palette();
            // palette.background.strong

            // container::style(Pair {
            //     color: color!(118, 150, 86).into(),
            //     text: palette.text,
            // })
            let green = Background::Color(color!(118, 150, 86)).into();
            let off_white = Background::Color(color!(238, 238, 210)).into();

            let (x, y) = (pos / 8, pos % 8);
            if (x + y) % 2 == 0 {
                Style {
                    background: green,
                    ..Style::default()
                }
            } else {
                Style {
                    background: off_white,
                    ..Style::default()
                }
            }

            // Style {
            //     background_color: color!(118, 150, 86).into(),
            //     ..Style::default()
            // }

            // let x = pos;

            // theme.style()

            // theme.theme(state, window)
            // match status {
            //     button::Status::Active => {
            //         button::Style::default()
            //            .with_background(palette.success.strong.color)
            //     }
            //     _ => button::primary(theme, status),
            // }
        };

        let (x, y) = (0, 0);

        let t01 = center(text(01)).style(theme_fn);
        let t02 = center(text(02)).style(theme_fn);
        let t03 = center(text(03)).style(theme_fn);
        let t04 = center(text(04)).style(theme_fn);
        let t05 = center(text(05)).style(theme_fn);
        let t06 = center(text(06)).style(theme_fn);
        let t07 = center(text(07)).style(theme_fn);
        let t08 = center(text(08)).style(theme_fn);
        let t09 = center(text(09)).style(theme_fn);
        let t10 = center(text(10)).style(theme_fn);
        let t11 = center(text(11)).style(theme_fn);
        let t12 = center(text(12)).style(theme_fn);
        let t13 = center(text(13)).style(theme_fn);
        let t14 = center(text(14)).style(theme_fn);
        let t15 = center(text(15)).style(theme_fn);
        let t16 = center(text(16)).style(theme_fn);
        let t17 = center(text(17)).style(theme_fn);
        let t18 = center(text(18)).style(theme_fn);
        let t19 = center(text(19)).style(theme_fn);
        let t20 = center(text(20)).style(theme_fn);
        let t21 = center(text(21)).style(theme_fn);
        let t22 = center(text(22)).style(theme_fn);
        let t23 = center(text(23)).style(theme_fn);
        let t24 = center(text(24)).style(theme_fn);
        let t25 = center(text(25)).style(theme_fn);
        let t26 = center(text(26)).style(theme_fn);
        let t27 = center(text(27)).style(theme_fn);
        let t28 = center(text(28)).style(theme_fn);
        let t29 = center(text(29)).style(theme_fn);
        let t30 = center(text(30)).style(theme_fn);
        let t31 = center(text(31)).style(theme_fn);
        let t32 = center(text(32)).style(theme_fn);
        let t33 = center(text(33)).style(theme_fn);
        let t34 = center(text(34)).style(theme_fn);
        let t35 = center(text(35)).style(theme_fn);
        let t36 = center(text(36)).style(theme_fn);
        let t37 = center(text(37)).style(theme_fn);
        let t38 = center(text(38)).style(theme_fn);
        let t39 = center(text(39)).style(theme_fn);
        let t40 = center(text(40)).style(theme_fn);
        let t41 = center(text(41)).style(theme_fn);
        let t42 = center(text(42)).style(theme_fn);
        let t43 = center(text(43)).style(theme_fn);
        let t44 = center(text(44)).style(theme_fn);
        let t45 = center(text(45)).style(theme_fn);
        let t46 = center(text(46)).style(theme_fn);
        let t47 = center(text(47)).style(theme_fn);
        let t48 = center(text(48)).style(theme_fn);
        let t49 = center(text(49)).style(theme_fn);
        let t50 = center(text(50)).style(theme_fn);
        let t51 = center(text(51)).style(theme_fn);
        let t52 = center(text(52)).style(theme_fn);
        let t53 = center(text(53)).style(theme_fn);
        let t54 = center(text(54)).style(theme_fn);
        let t55 = center(text(55)).style(theme_fn);
        let t56 = center(text(56)).style(theme_fn);
        let t57 = center(text(57)).style(theme_fn);
        let t58 = center(text(58)).style(theme_fn);
        let t59 = center(text(59)).style(theme_fn);
        let t60 = center(text(60)).style(theme_fn);
        let t61 = center(text(61)).style(theme_fn);
        let t62 = center(text(62)).style(theme_fn);
        let t63 = center(text(63)).style(theme_fn);
        let t64 = center(text(64)).style(theme_fn);

        grid!(
            t01, t02, t03, t04, t05, t06, t07, t08, t09, t10, t11, t12, t13, t14, t15, t16, t17,
            t18, t19, t20, t21, t22, t23, t24, t25, t26, t27, t28, t29, t30, t31, t32, t33, t34,
            t35, t36, t37, t38, t39, t40, t41, t42, t43, t44, t45, t46, t47, t48, t49, t50, t51,
            t52, t53, t54, t55, t56, t57, t58, t59, t60, t61, t62, t63, t64
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

    // pub fn grid_theme(pos: i32) -> Fn(&Theme) {
    //     let x = pos / 8;
    //     let y = pos % 8;
    //     let primary = container::primary;
    //     let secondary = container::secondary;
    //     if (x + y) % 2 == 0 {
    //         primary
    //     }
    //     else
    //     {
    //         secondary
    //     }
    // }

    // pub fn white_grid_theme(pos: i32) -> Fn(&Theme) {
    //     let x = pos / 8;
    //     let y = pos % 8;
    //     let primary = container::primary;
    //     let secondary = container::secondary;
    //     if (x + y) % 2 == 0 {
    //         primary
    //     }
    //     else
    //     {
    //         secondary
    //     }
    // }

    // pub fn green_grid_theme(pos: i32) -> Fn(&Theme) {
    //     let x = pos / 8;
    //     let y = pos % 8;
    //     let primary = container::primary;
    //     let secondary = container::secondary;
    //     if (x + y) % 2 == 0 {
    //         primary
    //     }
    //     else
    //     {
    //         secondary
    //     }
    // }
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
