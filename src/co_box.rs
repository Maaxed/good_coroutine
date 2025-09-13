use super::*;

pub type CoBox<Ctx> = Box<dyn DynCoroutine<Ctx>>;

pub fn co_box<Ctx, Co, M>(coroutine: Co) -> CoBox<Ctx>
where
	Co: IntoCoroutine<Ctx, M>,
	Co::Coroutine: 'static,
{
	Box::new(DynCoroutineImpl(coroutine.into_coroutine()))
}

pub trait DynCoroutine<Ctx>
{
	fn resume_dyn(self: Box<Self>, ctx: &mut Ctx) -> CoResult<CoBox<Ctx>>;
}

impl<Ctx> Coroutine<Ctx> for CoBox<Ctx>
{
	fn resume(self, ctx: &mut Ctx) -> CoResult<Self>
	{
		self.resume_dyn(ctx)
	}
}


struct DynCoroutineImpl<T>(T);

impl<Ctx, Co> DynCoroutine<Ctx> for DynCoroutineImpl<Co>
where
	Co: Coroutine<Ctx> + 'static,
{
	fn resume_dyn(mut self: Box<Self>, ctx: &mut Ctx) -> CoResult<CoBox<Ctx>>
	{
		match self.0.resume(ctx)
		{
			CoResult::Stop => CoResult::Stop,
			CoResult::RunNextFrame(co) =>
			{
				self.0 = co; // Reuse the same Box to avoid new allocation
				CoResult::RunNextFrame(self)
			},
		}
	}
}
