use super::*;

pub type CoBox<Ctx, Output> = Box<dyn DynCoroutine<Ctx, Output>>;

pub fn co_box<Ctx, Co, M>(coroutine: Co) -> CoBox<Ctx, <Co::Coroutine as Coroutine<Ctx, ()>>::Output>
where
	Co: IntoCoroutine<Ctx, (), M>,
	Co::Coroutine: 'static,
	<Co::Coroutine as Coroutine<Ctx, ()>>::State: 'static,
{
	Box::new(DynCoroutineImpl::Fn(coroutine.into_coroutine()))
}

pub trait DynCoroutine<Ctx, Output>
{
	fn resume_dyn(self: Box<Self>, ctx: &mut Ctx) -> CoResult<CoBox<Ctx, Output>, Output>;
}

impl<Ctx, Output> Coroutine<Ctx, ()> for CoBox<Ctx, Output>
{
	type Output = Output;
	type State = Self;

	fn init(self, ctx: &mut Ctx, _input: ()) -> CoResult<Self, Self::Output>
	{
		self.resume_dyn(ctx)
	}
}

impl<Ctx, Output> CoroutineState<Ctx> for CoBox<Ctx, Output>
{
	type Output = Output;

	fn resume(self, ctx: &mut Ctx) -> CoResult<Self, Self::Output>
	{
		self.resume_dyn(ctx)
	}
}


enum DynCoroutineImpl<C, CS>
{
	Fn(C),
	State(CS),
}

impl<Ctx, C, Output> DynCoroutine<Ctx, Output> for DynCoroutineImpl<C, C::State>
where
	C: Coroutine<Ctx, (), Output = Output> + 'static,
	C::State: 'static,
{
	fn resume_dyn(mut self: Box<Self>, ctx: &mut Ctx) -> CoResult<CoBox<Ctx, Output>, Output>
	{
		let res = match *self
		{
			Self::Fn(co) => co.init(ctx, ()),
			Self::State(co) => co.resume(ctx),
		};

		match res
		{
			CoResult::Stop(res) => CoResult::Stop(res),
			CoResult::RunNextFrame(co) =>
			{
				*self = DynCoroutineImpl::State(co); // Reuse the same Box to avoid new allocation
				CoResult::RunNextFrame(self)
			},
		}
	}
}
