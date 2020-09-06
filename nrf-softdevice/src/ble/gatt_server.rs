#[derive(defmt::Format)]
pub(crate) enum Event {
    Write,
    RwAuthorizeRequest,
    SysAttrMissing,
    Hvc,
    ScConfirm,
    ExchangeMtuRequest,
    Timeout,
    HvnTxComplete,
}

pub(crate) fn on_evt(evt: Event) {}
