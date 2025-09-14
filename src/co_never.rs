use super::*;


pub enum CoNever
{ }

impl<Ctx> CoroutineState<Ctx> for CoNever
{
	type Output = ();

	fn resume(self, _ctx: &mut Ctx) -> crate::CoResult<Self, Self::Output>
	{
		co_return(())
	}
}
