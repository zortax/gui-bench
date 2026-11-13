//! The orders: the header with the column titles, and one row per order. The app shows them as
//! a card with a plain `column` of every shown order, all real widgets; the list scenario reuses
//! the header and the row. A row is a button that tracks hover by itself, so its hover color
//! needs no message. The progress bar is iced's `progress_bar`.

use std::borrow::Cow;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, progress_bar, row, space, text};
use iced::{Background, Border, Element, Length, Shadow, border, padding};

use super::Message;
use super::sidebar::avatar;
use super::spec::{self, Order};
use super::style::{self, SEMIBOLD, color};

/// The column widths, from the left padding of the card to the amount.
const ORDER_WIDTH: f32 = spec::COL_CUSTOMER - spec::COL_ORDER;
const CUSTOMER_WIDTH: f32 = spec::COL_STATUS - spec::COL_CUSTOMER;
const STATUS_WIDTH: f32 = spec::COL_PROGRESS - spec::COL_STATUS;
/// The space between the avatar and the customer name.
const NAME_GAP: f32 = 40.0 - spec::AVATAR;
const PILL_HEIGHT: f32 = 22.0;

/// Returns the orders card with the orders of `orders` that pass the status checkboxes `shown`
/// and the search text. The direct edition builds every row from the state on each view; the
/// tuned edition wraps the card in iced's `lazy`, which rebuilds it only when the filters or
/// the sorting change.
pub fn view<'a>(
    orders: &'a [Order],
    shown: [bool; 4],
    search: &'a str,
    sorted: bool,
) -> Element<'a, Message> {
    let passing = move || {
        orders
            .iter()
            .filter(move |order| spec::visible(order, &shown, search))
    };
    #[cfg(feature = "tuned")]
    return iced::widget::lazy((shown, search, sorted), move |_| {
        card(passing().cloned().map(Cow::Owned), sorted)
    })
    .into();
    #[cfg(not(feature = "tuned"))]
    card(passing().map(Cow::Borrowed), sorted)
}

/// Returns the card: the header and the rows.
fn card<'a>(orders: impl Iterator<Item = Cow<'a, Order>>, sorted: bool) -> Element<'a, Message> {
    let mut orders = orders.peekable();
    let mut rows = column![header(sorted, Message::Sort, spec::CARD_RADIUS)];
    while let Some(order) = orders.next() {
        rows = rows.push(order_row(order, orders.peek().is_none(), Message::Order));
    }
    container(rows)
        .width(Length::Fill)
        .style(style::card)
        .into()
}

/// Returns the header: the column titles above a 1 px rule, its top corners rounded by `radius`.
/// The amount title is a button that sends `on_sort`, and takes the primary color while the
/// orders are `sorted`.
pub fn header<'a, M: Clone + 'a>(sorted: bool, on_sort: M, radius: f32) -> Element<'a, M> {
    let title = |label: &'static str| {
        text(label)
            .size(12)
            .font(SEMIBOLD)
            .color(color(spec::MUTED))
    };
    let [order, customer, status, progress, amount] = spec::COLUMNS;
    let sort = button(
        text(amount)
            .size(12)
            .font(SEMIBOLD)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Right)
            .align_y(Vertical::Center),
    )
    .padding(0)
    .width(spec::SORT_WIDTH)
    .height(Length::Fill)
    .on_press(on_sort)
    .style(move |_, _| button::Style {
        background: None,
        text_color: color(if sorted { spec::PRIMARY } else { spec::MUTED }),
        ..button::Style::default()
    });
    let titles = row![
        title(order).width(ORDER_WIDTH),
        title(customer).width(CUSTOMER_WIDTH),
        title(status).width(STATUS_WIDTH),
        title(progress),
        space::horizontal(),
        sort,
    ]
    .align_y(Vertical::Center)
    .padding(padding::horizontal(spec::COL_ORDER))
    .height(spec::TABLE_HEADER_HEIGHT - 1.0);
    let rule = container(space())
        .width(Length::Fill)
        .height(1)
        .style(|_| style::filled(spec::RULE, 0));
    container(column![titles, rule])
        .width(Length::Fill)
        .style(move |_| style::filled(spec::TABLE_HEADER_BG, border::top(radius)))
        .into()
}

/// Returns the row of an order, borrowed from the state or owned by the row. A press sends
/// `on_press`; the last row of a card rounds its bottom corners like the card.
pub fn order_row<'a, M: Clone + 'a>(
    order: Cow<'a, Order>,
    last: bool,
    on_press: M,
) -> Element<'a, M> {
    let (pill_bg, pill_text) = order.status.colors();
    let (number, amount) = (order.number_text(), order.amount());
    let (status, progress, avatar_color) = (order.status, order.progress, order.avatar);
    let (initials, name): (Cow<'a, str>, Cow<'a, str>) = match order {
        Cow::Borrowed(order) => (
            Cow::Borrowed(&order.initials),
            Cow::Borrowed(&order.customer),
        ),
        Cow::Owned(order) => (Cow::Owned(order.initials), Cow::Owned(order.customer)),
    };
    let customer = row![
        avatar(initials, spec::AVATAR, 11.0, avatar_color),
        text(name).size(spec::FONT_SIZE).color(color(spec::TEXT)),
    ]
    .spacing(NAME_GAP)
    .align_y(Vertical::Center)
    .width(CUSTOMER_WIDTH);
    let pill = container(
        text(status.label())
            .size(12)
            .font(SEMIBOLD)
            .color(color(pill_text)),
    )
    .center_y(PILL_HEIGHT)
    .padding([0, 10])
    .style(move |_| style::filled(pill_bg, PILL_HEIGHT / 2.0));
    let progress = progress_bar(0.0..=100.0, progress as f32)
        .length(spec::PROGRESS_WIDTH)
        .girth(spec::PROGRESS_HEIGHT)
        .style(|_| progress_bar::Style {
            background: Background::Color(color(spec::TRACK)),
            bar: Background::Color(color(spec::PRIMARY)),
            border: border::rounded(spec::PROGRESS_HEIGHT / 2.0),
        });
    let cells = row![
        text(number)
            .size(spec::FONT_SIZE)
            .color(color(spec::MUTED))
            .width(ORDER_WIDTH),
        customer,
        container(pill).width(STATUS_WIDTH),
        progress,
        text(amount)
            .size(spec::FONT_SIZE)
            .font(SEMIBOLD)
            .color(color(spec::TEXT))
            .width(Length::Fill)
            .align_x(Horizontal::Right),
    ]
    .align_y(Vertical::Center)
    .padding(padding::horizontal(spec::COL_ORDER))
    .height(spec::ORDER_ROW_HEIGHT - 1.0);
    let rule = container(space())
        .width(Length::Fill)
        .height(1)
        .style(|_| style::filled(spec::ROW_RULE, 0));
    let radius = if last {
        border::bottom(spec::CARD_RADIUS)
    } else {
        border::radius(0)
    };
    button(column![cells, rule])
        .padding(0)
        .width(Length::Fill)
        .height(spec::ORDER_ROW_HEIGHT)
        .on_press(on_press)
        .style(move |_, status| {
            let background = match status {
                button::Status::Hovered | button::Status::Pressed => {
                    Some(Background::Color(color(spec::ROW_HOVER)))
                }
                button::Status::Active | button::Status::Disabled => None,
            };
            button::Style {
                background,
                text_color: color(spec::TEXT),
                border: Border::default().rounded(radius),
                shadow: Shadow::default(),
                snap: true,
            }
        })
        .into()
}
