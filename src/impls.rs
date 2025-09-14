use variadics_please::all_tuples_enumerated;

use super::*;


#[doc(hidden)]
pub struct CoSelfMarker;

impl<Ctx, Input, T> IntoCoroutine<Ctx, Input, (CoSelfMarker, T)> for T
where
	T: Coroutine<Ctx, Input>,
{
	type Coroutine = Self;

	fn into_coroutine(self) -> Self::Coroutine
	{
		self
	}
}


#[doc(hidden)]
pub struct CoTupleMarker;

macro_rules! impl_coroutine_tuple
{
	(($i_head:tt, $sys_head:ident, $marker_head:ident) $(, ($i_tail:tt, $sys_tail:ident, $marker_tail:ident))*) =>
	{
		impl<Ctx, Input, $sys_head, $marker_head, $($sys_tail, $marker_tail),*> IntoCoroutine<Ctx, Input, (CoTupleMarker, $marker_head, $($marker_tail,)*)> for ($sys_head, $($sys_tail,)*)
		where
			$sys_head: IntoCoroutine<Ctx, Input, $marker_head>,
			($($sys_tail,)*): IntoCoroutine<Ctx, <<$sys_head as IntoCoroutine<Ctx, Input, $marker_head>>::Coroutine as Coroutine<Ctx, Input>>::Output, (CoTupleMarker, $($marker_tail,)*)>
		{
			type Coroutine = CoChain<
				<$sys_head as IntoCoroutine<Ctx, Input, $marker_head>>::Coroutine,
				<($($sys_tail,)*) as IntoCoroutine<Ctx, <<$sys_head as IntoCoroutine<Ctx, Input, $marker_head>>::Coroutine as Coroutine<Ctx, Input>>::Output, (CoTupleMarker, $($marker_tail,)*)>>::Coroutine
			>;

			fn into_coroutine(self) -> Self::Coroutine
			{
				CoChain::new(self.$i_head.into_coroutine(), ($(self.$i_tail,)*).into_coroutine())
			}
		}
	}
}

impl<Ctx, Input, T, M> IntoCoroutine<Ctx, Input, (CoTupleMarker, M,)> for (T,)
where
	T: IntoCoroutine<Ctx, Input, M>,
{
	type Coroutine = T::Coroutine;

	fn into_coroutine(self) -> Self::Coroutine
	{
		self.0.into_coroutine()
	}
}

all_tuples_enumerated!(impl_coroutine_tuple, 2, 10, T, M);
