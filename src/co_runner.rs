use super::*;

pub struct CoroutineRunner<Ctx>
{
	coroutines: Vec<Option<CoBox<Ctx, ()>>>,
}

impl<Ctx> Default for CoroutineRunner<Ctx>
{
	fn default() -> Self
	{
		Self
		{
			coroutines: Vec::new(),
		}
	}
}

impl<Ctx> CoroutineRunner<Ctx>
{
	pub fn new() -> Self
	{
		Self::default()
	}

	pub fn is_empty(&self) -> bool
	{
		self.coroutines.is_empty()
	}

	pub fn resume(&mut self, ctx: &mut Ctx)
	{
		self.coroutines.retain_mut(|co_box|
		{
			let co = co_box.take().unwrap();

			match co.resume(ctx)
			{
				CoResult::Stop(()) => false,
				CoResult::RunNextFrame(new_co) =>
				{
					*co_box = Some(new_co);
					true
				}
			}
		});
	}

	pub fn push<C>(&mut self, co: C)
	where
		C: Coroutine<Ctx, (), Output = ()> + 'static,
		C::State: 'static,
	{
		self.push_boxed(co_box(co));
	}

	pub fn push_boxed(&mut self, co: CoBox<Ctx, ()>)
	{
		self.coroutines.push(Some(co))
	}
}
