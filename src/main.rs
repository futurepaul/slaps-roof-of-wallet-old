use druid_shell::platform::WindowBuilder;
use druid_shell::win_main;

use druid::widget::{Column, Label, Padding};
use druid::{UiMain, UiState};

use druid::Id;

fn pad(widget: Id, state: &mut UiState) -> Id {
    Padding::uniform(5.0).ui(widget, state)
}

pub mod lnd;
pub mod wallet_widgets;

fn main() {
    druid_shell::init();

    let mut run_loop = win_main::RunLoop::new();
    let mut builder = WindowBuilder::new();
    let mut state = UiState::new();

    let creds = lnd::Credentials::read_from_env();
    let client = lnd::new_client(creds);

    //ugh this is wrong to do this twice
    let creds = lnd::Credentials::read_from_env();

    let wallet_info = lnd::get_info(&creds, &client);
    dbg!(&wallet_info);
    let title = pad(Label::new(wallet_info.alias).ui(&mut state), &mut state);

    let balance_msg = format!(
        "Balance (on chain): {} sats",
        lnd::get_wallet_balance(&creds, &client)
    );
    let balance_label = Label::new(balance_msg).ui(&mut state);
    let balance_padded = pad(balance_label, &mut state);

    let channel_msg = format!(
        "Balance (in channels): {} sats",
        lnd::get_channel_balance(&creds, &client)
    );
    let channel_label = Label::new(channel_msg).ui(&mut state);
    let channel_padded = pad(channel_label, &mut state);

    let qr = pad(
        wallet_widgets::Qr::new(wallet_info.identity_pubkey.to_string()).ui(&mut state),
        &mut state,
    );

    let column = Column::new();
    let panel = column.ui(&[title, balance_padded, channel_padded, qr], &mut state);

    state.set_root(panel);
    builder.set_handler(Box::new(UiMain::new(state)));
    builder.set_title("Slaps Roof Of Wallet");
    let window = builder.build().expect("built window");
    window.show();
    run_loop.run();
}
