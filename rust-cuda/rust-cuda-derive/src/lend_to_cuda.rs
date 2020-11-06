use proc_macro::TokenStream;
use quote::quote;

use super::generics;

#[allow(clippy::module_name_repetitions)]
pub fn impl_lend_to_cuda(ast: &syn::DeriveInput) -> TokenStream {
    if !matches!(ast.data, syn::Data::Struct(_)) {
        abort_call_site!("You can only derive the `LendToCuda` trait on structs for now.");
    };

    let struct_name = &ast.ident;

    let (_r2c_attrs, r2c_generics) =
        generics::expand_cuda_struct_generics_where_requested_in_attrs(ast);
    let (impl_generics, ty_generics, where_clause) = r2c_generics.split_for_impl();

    (quote! {
        #[cfg(not(target_os = "cuda"))]
        unsafe impl #impl_generics rust_cuda::host::LendToCuda for #struct_name #ty_generics
            #where_clause
        {
            fn lend_to_cuda<
                O,
                LendToCudaInnerFunc: FnOnce(
                    rustacuda_core::DevicePointer<
                        <Self as rust_cuda::common::RustToCuda>::CudaRepresentation
                    >
                ) -> rustacuda::error::CudaResult<O>,
            >(
                &self,
                inner: LendToCudaInnerFunc,
            ) -> rustacuda::error::CudaResult<O> {
                use rust_cuda::common::RustToCuda;

                let (cuda_repr, tail_alloc) = unsafe {
                    self.borrow(rust_cuda::host::NullCudaAlloc)
                }?;

                let mut device_box = rust_cuda::host::CudaDropWrapper::from(
                    rustacuda::memory::DeviceBox::new(&cuda_repr)?
                );
                let cuda_ptr = device_box.as_device_ptr();

                let alloc = rust_cuda::host::CombinedCudaAlloc::new(device_box, tail_alloc);

                let result = inner(cuda_ptr);

                core::mem::drop(alloc);

                result
            }

            fn lend_to_cuda_mut<
                O,
                LendToCudaInnerFunc: FnOnce(
                    rustacuda_core::DevicePointer<
                        <Self as rust_cuda::common::RustToCuda>::CudaRepresentation
                    >
                ) -> rustacuda::error::CudaResult<O>,
            >(
                &mut self,
                inner: LendToCudaInnerFunc,
            ) -> rustacuda::error::CudaResult<O> {
                self.lend_to_cuda(inner)
            }
        }

        #[cfg(target_os = "cuda")]
        unsafe impl #impl_generics rust_cuda::device::BorrowFromRust for #struct_name #ty_generics
            #where_clause
        {
            unsafe fn with_borrow_from_rust_mut<O, LendToCudaInnerFunc: FnOnce(
                &mut Self
            ) -> O>(
                cuda_repr_ptr: *mut <Self as rust_cuda::common::RustToCuda>::CudaRepresentation,
                inner: LendToCudaInnerFunc,
            ) -> O {
                use rust_cuda::common::CudaAsRust;

                let cuda_repr_ref: &mut <
                    Self as rust_cuda::common::RustToCuda
                >::CudaRepresentation = &mut *cuda_repr_ptr;

                let mut rust_repr = cuda_repr_ref.as_rust();

                let result = inner(&mut rust_repr);

                // MUST forget about rust_repr as we do NOT own any of the heap memory
                // it might reference
                core::mem::forget(rust_repr);

                result
            }

            unsafe fn with_borrow_from_rust<O, LendToCudaInnerFunc: FnOnce(
                &Self
            ) -> O>(
                cuda_repr_ptr: *const <Self as rust_cuda::common::RustToCuda>::CudaRepresentation,
                inner: LendToCudaInnerFunc,
            ) -> O {
                // The cast from *const to *mut is only safe because &mut is
                // restricted to & to the caller
                Self::with_borrow_from_rust_mut(
                    cuda_repr_ptr as *mut <
                        Self as rust_cuda::common::RustToCuda
                    >::CudaRepresentation,
                    |mut_ref| inner(&*mut_ref),
                )
            }
        }
    })
    .into()
}
