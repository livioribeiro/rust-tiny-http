use super::{Request, Response};

pub trait Handler {
    fn handle(&self, req: Request, res: Response);
}
