use crate::util::*;
use ndarray::prelude::*;

#[derive(Debug)]
pub enum ArrayOut<'a, F, D>
where
    D: Dimension,
{
    ViewMut(ArrayViewMut<'a, F, D>),
    Owned(Array<F, D>),
    ToBeCloned(ArrayViewMut<'a, F, D>, Array<F, D>),
}

impl<F, D> ArrayOut<'_, F, D>
where
    F: Clone,
    D: Dimension,
{
    pub fn view(&self) -> ArrayView<'_, F, D> {
        match self {
            Self::ViewMut(arr) => arr.view(),
            Self::Owned(arr) => arr.view(),
            Self::ToBeCloned(_, arr) => arr.view(),
        }
    }

    pub fn view_mut(&mut self) -> ArrayViewMut<'_, F, D> {
        match self {
            Self::ViewMut(arr) => arr.view_mut(),
            Self::Owned(arr) => arr.view_mut(),
            Self::ToBeCloned(_, arr) => arr.view_mut(),
        }
    }

    pub fn into_owned(self) -> Array<F, D> {
        match self {
            Self::ViewMut(arr) => arr.to_owned(),
            Self::Owned(arr) => arr,
            Self::ToBeCloned(mut arr_view, arr_owned) => {
                arr_view.assign(&arr_owned);
                arr_owned
            },
        }
    }

    pub fn is_view_mut(&mut self) -> bool {
        match self {
            Self::ViewMut(_) => true,
            Self::Owned(_) => false,
            Self::ToBeCloned(_, _) => true,
        }
    }

    pub fn is_owned(&mut self) -> bool {
        match self {
            Self::ViewMut(_) => false,
            Self::Owned(_) => true,
            Self::ToBeCloned(_, _) => false,
        }
    }

    pub fn clone_to_view_mut(self) -> Self {
        match self {
            ArrayOut::ToBeCloned(mut arr_view, arr_owned) => {
                arr_view.assign(&arr_owned);
                ArrayOut::ViewMut(arr_view)
            },
            _ => self,
        }
    }

    pub fn reversed_axes(self) -> Self {
        match self {
            ArrayOut::ViewMut(arr) => ArrayOut::ViewMut(arr.reversed_axes()),
            ArrayOut::Owned(arr) => ArrayOut::Owned(arr.reversed_axes()),
            ArrayOut::ToBeCloned(mut arr_view, arr_owned) => {
                arr_view.assign(&arr_owned);
                ArrayOut::ViewMut(arr_view.reversed_axes())
            },
        }
    }
}

pub type ArrayOut1<'a, F> = ArrayOut<'a, F, Ix1>;
pub type ArrayOut2<'a, F> = ArrayOut<'a, F, Ix2>;
pub type ArrayOut3<'a, F> = ArrayOut<'a, F, Ix3>;

/* #endregion */

/* #region Strides */

#[inline]
pub fn get_layout_array2<F>(arr: &ArrayView2<F>) -> BLASLayout {
    // Note that this only shows order of matrix (dimension information)
    // not c/f-contiguous (memory layout)
    // So some sequential (both c/f-contiguous) cases may be considered as only row or col major
    // Examples:
    // RowMajor     ==>   shape=[1, 4], strides=[0, 1], layout=CFcf (0xf)
    // ColMajor     ==>   shape=[4, 1], strides=[1, 0], layout=CFcf (0xf)
    // Sequential   ==>   shape=[1, 1], strides=[0, 0], layout=CFcf (0xf)
    // NonContig    ==>   shape=[4, 1], strides=[10, 0], layout=Custom (0x0)
    let (d0, d1) = arr.dim();
    let [s0, s1] = arr.strides().try_into().unwrap();
    if d0 == 0 || d1 == 0 {
        // empty array
        return BLASLayout::Sequential;
    } else if d0 == 1 && d1 == 1 {
        // one element
        return BLASLayout::Sequential;
    } else if s1 == 1 {
        // row-major
        return BLASRowMajor;
    } else if s0 == 1 {
        // col-major
        return BLASColMajor;
    } else {
        // non-contiguous
        return BLASLayout::NonContiguous;
    }
}

/* #endregion */

/* #region flip */

pub(crate) fn flip_trans_fpref<'a, F>(
    trans: BLASTranspose,
    view: &'a ArrayView2<F>,
    view_t: &'a ArrayView2<F>,
    hermi: bool,
) -> Result<(BLASTranspose, CowArray<'a, F, Ix2>), BLASError>
where
    F: BLASFloat,
{
    match (get_layout_array2(&view).is_fpref(), trans) {
        (true, _) => Ok((trans, view_t.as_standard_layout())),
        (false, BLASNoTrans) => Ok((
            trans.flip(hermi),
            match hermi {
                false => view.as_standard_layout(),
                true => CowArray::from(view.mapv(F::conj)),
            },
        )),
        (false, BLASTrans) => Ok((trans.flip(hermi), view.as_standard_layout())),
        (false, BLASConjTrans) => Ok((trans.flip(hermi), CowArray::from(view.mapv(F::conj)))),
        (false, trans) => blas_invalid!(trans),
    }
}

pub(crate) fn flip_trans_cpref<'a, F>(
    trans: BLASTranspose,
    view: &'a ArrayView2<F>,
    view_t: &'a ArrayView2<F>,
    hermi: bool,
) -> Result<(BLASTranspose, CowArray<'a, F, Ix2>), BLASError>
where
    F: BLASFloat,
{
    match (get_layout_array2(&view).is_cpref(), trans) {
        (true, _) => Ok((trans, view.as_standard_layout())),
        (false, BLASNoTrans) => Ok((
            trans.flip(hermi),
            match hermi {
                false => view_t.as_standard_layout(),
                true => CowArray::from(view_t.mapv(F::conj)),
            },
        )),
        (false, BLASTrans) => Ok((trans.flip(hermi), view_t.as_standard_layout())),
        (false, BLASConjTrans) => Ok((trans.flip(hermi), CowArray::from(view_t.mapv(F::conj)))),
        (false, trans) => blas_invalid!(trans),
    }
}

/* #endregion */
