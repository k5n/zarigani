use actix::prelude::*;

#[derive(Message)]
#[rtype(usize)]
struct Sum(usize, usize);

// Actor definition
struct Calculator;

impl Actor for Calculator {
    type Context = Context<Self>;
}

// now we need to implement `Handler` on `Calculator` for the `Sum` message.
impl Handler<Sum> for Calculator {
    type Result = usize; // <- Message response type

    fn handle(&mut self, msg: Sum, _ctx: &mut Context<Self>) -> Self::Result {
        msg.0 + msg.1
    }
}

#[actix::main]
async fn main() {
    let addr = Calculator.start();
    let res = addr.send(Sum(10, 5)).await;

    match res {
        Ok(result) => println!("SUM: {}", result),
        _ => println!("Communication to the actor has failed"),
    }
}
