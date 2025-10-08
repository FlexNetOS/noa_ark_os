use noa_abi::Budget;

pub trait Broker {
    fn request(&mut self, task: &str, need: Budget) -> bool;
    fn refund(&mut self, task: &str, give: Budget);
}

pub struct FixedBroker { remain: Budget }
impl FixedBroker { pub fn new(remain: Budget) -> Self { Self { remain } } }
impl Broker for FixedBroker {
    fn request(&mut self, _task: &str, need: Budget) -> bool {
        if need.ms<=self.remain.ms && need.tokens<=self.remain.tokens && need.io<=self.remain.io {
            self.remain.ms-=need.ms; self.remain.tokens-=need.tokens; self.remain.io-=need.io; true
        } else { false }
    }
    fn refund(&mut self, _task: &str, give: Budget) {
        self.remain.ms+=give.ms; self.remain.tokens+=give.tokens; self.remain.io+=give.io;
    }
}
