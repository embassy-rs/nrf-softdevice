#[derive(defmt::Format)]
pub(crate) enum Event {
    PrimSrvcDiscRsp,
    RelDiscRsp,
    CharDiscRsp,
    DescDiscRsp,
    AttrInfoDiscRsp,
    CharValByUuidReadRsp,
    ReadRsp,
    CharValsReadRsp,
    WriteRsp,
    Hvx,
    ExchangeMtuRsp,
    Timeout,
    WriteCmdTxComplete,
}

pub(crate) fn on_evt(evt: Event) {}
