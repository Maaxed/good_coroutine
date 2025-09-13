use super::*;


pub enum CoNever
{ }

impl<Ctx> Coroutine<Ctx> for CoNever
{
	fn resume(self, _ctx: &mut Ctx) -> crate::CoResult<Self>
	{
		co_return()
	}
}
