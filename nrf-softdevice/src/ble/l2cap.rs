#[derive(defmt::Format)]
pub(crate) enum Event {
    ChSetupRequest,
    ChSetupRefused,
    ChSetup,
    ChReleased,
    ChSduBufReleased,
    ChCredit,
    ChRx,
    ChTx,
}

pub(crate) fn on_evt(evt: Event) {}
