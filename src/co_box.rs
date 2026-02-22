use super::*;

pub type CoBox<Ctx, Output = ()> = Box<dyn DynCoroutine<Ctx, Output>>;

pub fn co_box<Ctx, C>(coroutine: C) -> CoBox<Ctx, C::Output>
where
	C: Coroutine<Ctx> + 'static,
{
	Box::new(DynCoroutineImpl(coroutine))
}

pub trait DynCoroutine<Ctx, Output>
{
	fn resume_dyn(self: Box<Self>, ctx: &mut Ctx) -> CoResult<CoBox<Ctx, Output>, Output>;
}

impl<Ctx, Output> Coroutine<Ctx> for CoBox<Ctx, Output>
{
	type Output = Output;

	fn resume(self, ctx: &mut Ctx) -> CoResult<Self, Self::Output>
	{
		self.resume_dyn(ctx)
	}
}


struct DynCoroutineImpl<C>(C);

impl<Ctx, C, Output> DynCoroutine<Ctx, Output> for DynCoroutineImpl<C>
where
	C: Coroutine<Ctx, Output = Output> + 'static,
{
	fn resume_dyn(mut self: Box<Self>, ctx: &mut Ctx) -> CoResult<CoBox<Ctx, Output>, Output>
	{
		match self.0.resume(ctx)
		{
			CoResult::Stop(res) => CoResult::Stop(res),
			CoResult::RunNextFrame(co) =>
			{
				self.0 = co; // Reuse the same Box to avoid new allocation
				CoResult::RunNextFrame(self)
			},
		}
	}
}
