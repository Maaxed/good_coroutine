use super::*;
use variadics_please::all_tuples_enumerated;


pub fn co_concurrent<T>(t: T) -> CoConcurrent<T>
{
	IgnoreOutput(co_concurrent_with_output(t))
}

pub fn co_concurrent_with_output<T>(t: T) -> CoConcurrentWithOutput<T>
{
	CoConcurrentWithOutput(t)
}


pub type CoConcurrent<T> = IgnoreOutput<CoConcurrentWithOutput<T>>;
pub type CoConcurrentState<T> = IgnoreOutput<CoConcurrentStateWithOutput<T>>;

pub struct CoConcurrentWithOutput<T>(T);
pub struct CoConcurrentStateWithOutput<T>(T);


mod array
{
	use super::*;

	impl<Ctx, C, Input, const N: usize> Coroutine<Ctx, [Input; N]> for CoConcurrentWithOutput<[C; N]>
	where
		C: Coroutine<Ctx, Input>
	{
		type Output = [C::Output; N];
		type State = CoConcurrentStateWithOutput<[CoResult<C::State, C::Output>; N]>;
	
		fn init(self, ctx: &mut Ctx, input: [Input; N]) -> CoResult<Self::State, Self::Output>
		{
			let mut co = self.0.into_iter();
			let mut input = input.into_iter();
	
			let state = CoConcurrentStateWithOutput(
				std::array::from_fn(|_| co.next().unwrap().init(ctx, input.next().unwrap()))
			);
	
			state.check_completion()
		}
	}
	
	impl<Ctx, C, const N: usize> Coroutine<Ctx, ()> for CoConcurrentWithOutput<[C; N]>
	where
		C: Coroutine<Ctx, ()>
	{
		type Output = [C::Output; N];
		type State = CoConcurrentStateWithOutput<[CoResult<C::State, C::Output>; N]>;
	
		fn init(self, ctx: &mut Ctx, _input: ()) -> CoResult<Self::State, Self::Output>
		{
			self.init(ctx, [(); N])
		}
	}
	
	impl<Ctx, C, const N: usize> CoroutineState<Ctx> for CoConcurrentStateWithOutput<[CoResult<C, C::Output>; N]>
	where
		C: CoroutineState<Ctx>
	{
		type Output = [C::Output; N];
	
		fn resume(self, ctx: &mut Ctx) -> CoResult<Self, Self::Output>
		{
			let state = Self(
				self.0.map(|res|
					match res
					{
						CoResult::Stop(res) => CoResult::Stop(res),
						CoResult::RunNextFrame(co) => co.resume(ctx),
					}
				)
			);
			
			state.check_completion()
		}
	}
	
	impl<C, Output, const N: usize> CoConcurrentStateWithOutput<[CoResult<C, Output>; N]>
	{
		fn check_completion<Ctx>(self) -> CoResult<Self, [Output; N]>
		where
			C: CoroutineState<Ctx, Output = Output>
		{
			if self.0.iter().all(|res| matches!(res, CoResult::Stop(_)))
			{
				CoResult::Stop(self.0.map(|res|
					match res
					{
						CoResult::Stop(res) => res,
						_ => unreachable!(),
					}
				))
			}
			else
			{
				CoResult::RunNextFrame(self)
			}
		}
	}
}

macro_rules! impl_co_concurrent_tuple
{
	($(($i:tt, $C:ident, $Input:ident, $Output:ident, $res:ident)),*) =>
	{
		impl<Ctx, $($C, $Input),*> Coroutine<Ctx, ($($Input,)*)> for CoConcurrentWithOutput<($($C,)*)>
		where
			$( $C: Coroutine<Ctx, $Input>, )*
		{
			type Output = (
				$( $C::Output, )*
			);
			type State = CoConcurrentStateWithOutput<(
				$( CoResult<$C::State, $C::Output>, )*
			)>;

			fn init(self, ctx: &mut Ctx, input: ($($Input,)*)) -> CoResult<Self::State, Self::Output>
			{
				let state = CoConcurrentStateWithOutput((
					$( self.0.$i.init(ctx, input.$i), )*
				));
				
				state.check_completion()
			}
		}

		impl<Ctx, $($C),*> Coroutine<Ctx, ()> for CoConcurrentWithOutput<($($C,)*)>
		where
			$( $C: Coroutine<Ctx, ()>, )*
		{
			type Output = (
				$( $C::Output, )*
			);
			type State = CoConcurrentStateWithOutput<(
				$( CoResult<$C::State, $C::Output>, )*
			)>;

			fn init(self, ctx: &mut Ctx, _input: ()) -> CoResult<Self::State, Self::Output>
			{
				self.init(ctx, ($({ let $res = (); $res },)*))
			}
		}

		impl<Ctx, $($C),*> CoroutineState<Ctx> for CoConcurrentStateWithOutput<($( CoResult<$C, $C::Output>,)*)>
		where
			$( $C: CoroutineState<Ctx>, )*
		{
			type Output = (
				$( $C::Output, )*
			);

			fn resume(self, ctx: &mut Ctx) -> CoResult<Self, Self::Output>
			{
				let state = Self((
					$(
						match self.0.$i
						{
							CoResult::Stop(res) => CoResult::Stop(res),
							CoResult::RunNextFrame(co) => co.resume(ctx),
						},
					)*
				));
				
				state.check_completion()
			}
		}

		impl<$($C, $Output),*> CoConcurrentStateWithOutput<($( CoResult<$C, $Output>,)*)>
		{
			fn check_completion<Ctx>(self) -> CoResult<Self, ($( $C::Output, )*)>
			where
				$( $C: CoroutineState<Ctx, Output = $Output>, )*
			{
				if let Self((
					$( CoResult::Stop($res), )*
				)) = self
				{
					CoResult::Stop((
						$( $res, )*
					))
				}
				else
				{
					CoResult::RunNextFrame(self)
				}
			}
		}
	}
}

all_tuples_enumerated!(impl_co_concurrent_tuple, 1, 10, C, Input, Output, res);


impl<Ctx, C, Input> Coroutine<Ctx, Vec<Input>> for CoConcurrentWithOutput<Vec<C>>
where
	C: Coroutine<Ctx, Input>,
{
	type Output = Vec<C::Output>;
	type State = CoConcurrentStateWithOutput<Vec<CoResult<C::State, C::Output>>>;

	fn init(self, ctx: &mut Ctx, input: Vec<Input>) -> CoResult<Self::State, Self::Output>
	{
		assert_eq!(self.0.len(), input.len());
		
		let state: Self::State = CoConcurrentStateWithOutput(
			self.0.into_iter().zip(input)
				.map(|(co, input)| co.init(ctx, input))
				.collect()
		);

		state.check_completion()
	}
}

impl<Ctx, C> Coroutine<Ctx, ()> for CoConcurrentWithOutput<Vec<C>>
where
	C: Coroutine<Ctx, ()>,
{
	type Output = Vec<C::Output>;
	type State = CoConcurrentStateWithOutput<Vec<CoResult<C::State, C::Output>>>;

	fn init(self, ctx: &mut Ctx, _input: ()) -> CoResult<Self::State, Self::Output>
	{
		let size = self.0.len();
		self.init(ctx, vec![(); size])
	}
}

impl<Ctx, C> CoroutineState<Ctx> for CoConcurrentStateWithOutput<Vec<CoResult<C, C::Output>>>
where
	C: CoroutineState<Ctx>
{
	type Output = Vec<C::Output>;

	fn resume(self, ctx: &mut Ctx) -> CoResult<Self, Self::Output>
	{
		let state = Self(
			self.0.into_iter().map(|res|
				match res
				{
					CoResult::Stop(res) => CoResult::Stop(res),
					CoResult::RunNextFrame(co) => co.resume(ctx),
				}
			).collect()
		);
		
		state.check_completion()
	}
}

impl<C, Output> CoConcurrentStateWithOutput<Vec<CoResult<C, Output>>>
{
	fn check_completion<Ctx>(self) -> CoResult<Self, Vec<Output>>
	where
		C: CoroutineState<Ctx, Output = Output>
	{
		if self.0.iter().all(|res| matches!(res, CoResult::Stop(_)))
		{
			CoResult::Stop(self.0.into_iter().map(|res|
				match res
				{
					CoResult::Stop(res) => res,
					_ => unreachable!(),
				}
			).collect())
		}
		else
		{
			CoResult::RunNextFrame(self)
		}
	}
}
