#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr::addr_of_mut;
use crate::types::*;
use crate::enums::*;

#[repr(C, u64)]
pub enum SetRenderTreeParametersComponentsEnum {
    TypeWindow(Window),
    TypeVerticalLayout(VerticalLayout),
    TypeHorizontalLayout(HorizontalLayout),
    TypeGridLayout(GridLayout),
    TypeGridLayoutColumn(GridLayoutColumn),
    TypeGridLayoutRow(GridLayoutRow),
    TypeButton(Button),
}

#[repr(C)]
struct SetRenderTreeParametersComponentsEnumStruct {
    tag: SetRenderTreeParametersComponentsEnumStructTag,
    payload: SetRenderTreeParametersComponentsEnumStructPayload,
}

#[repr(u64)]
enum SetRenderTreeParametersComponentsEnumStructTag {
    TypeWindow,
    TypeVerticalLayout,
    TypeHorizontalLayout,
    TypeGridLayout,
    TypeGridLayoutColumn,
    TypeGridLayoutRow,
    TypeButton,
}

#[repr(C)]
union SetRenderTreeParametersComponentsEnumStructPayload {
    payload_type_window: ManuallyDrop<Window>,
    payload_type_vertical_layout: ManuallyDrop<VerticalLayout>,
    payload_type_horizontal_layout: ManuallyDrop<HorizontalLayout>,
    payload_type_grid_layout: ManuallyDrop<GridLayout>,
    payload_type_grid_layout_column: ManuallyDrop<GridLayoutColumn>,
    payload_type_grid_layout_row: ManuallyDrop<GridLayoutRow>,
    payload_type_button: ManuallyDrop<Button>,
}

impl SetRenderTreeParametersComponentsEnum {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut SetRenderTreeParametersComponentsEnum, 1);
        pointer = pointer.offset(mem::size_of::<SetRenderTreeParametersComponentsEnum>() as isize);
        mem::size_of::<SetRenderTreeParametersComponentsEnum>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        match self {
            SetRenderTreeParametersComponentsEnum::TypeWindow(value) => {
                value.write_references_at(pointer)
            },
            SetRenderTreeParametersComponentsEnum::TypeVerticalLayout(value) => {
                value.write_references_at(pointer)
            },
            SetRenderTreeParametersComponentsEnum::TypeHorizontalLayout(value) => {
                value.write_references_at(pointer)
            },
            SetRenderTreeParametersComponentsEnum::TypeGridLayout(value) => {
                value.write_references_at(pointer)
            },
            SetRenderTreeParametersComponentsEnum::TypeGridLayoutColumn(value) => {
                value.write_references_at(pointer)
            },
            SetRenderTreeParametersComponentsEnum::TypeGridLayoutRow(value) => {
                value.write_references_at(pointer)
            },
            SetRenderTreeParametersComponentsEnum::TypeButton(value) => {
                value.write_references_at(pointer)
            },
        }
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut SetRenderTreeParametersComponentsEnum, references_pointer: *mut u8) -> usize {
        let object = object_pointer as *mut SetRenderTreeParametersComponentsEnumStruct;
        match (*object).tag {
            SetRenderTreeParametersComponentsEnumStructTag::TypeWindow => {
                Window::reconstruct_at(addr_of_mut!((*object).payload.payload_type_window) as *mut Window, references_pointer)
            },
            SetRenderTreeParametersComponentsEnumStructTag::TypeVerticalLayout => {
                VerticalLayout::reconstruct_at(addr_of_mut!((*object).payload.payload_type_vertical_layout) as *mut VerticalLayout, references_pointer)
            },
            SetRenderTreeParametersComponentsEnumStructTag::TypeHorizontalLayout => {
                HorizontalLayout::reconstruct_at(addr_of_mut!((*object).payload.payload_type_horizontal_layout) as *mut HorizontalLayout, references_pointer)
            },
            SetRenderTreeParametersComponentsEnumStructTag::TypeGridLayout => {
                GridLayout::reconstruct_at(addr_of_mut!((*object).payload.payload_type_grid_layout) as *mut GridLayout, references_pointer)
            },
            SetRenderTreeParametersComponentsEnumStructTag::TypeGridLayoutColumn => {
                GridLayoutColumn::reconstruct_at(addr_of_mut!((*object).payload.payload_type_grid_layout_column) as *mut GridLayoutColumn, references_pointer)
            },
            SetRenderTreeParametersComponentsEnumStructTag::TypeGridLayoutRow => {
                GridLayoutRow::reconstruct_at(addr_of_mut!((*object).payload.payload_type_grid_layout_row) as *mut GridLayoutRow, references_pointer)
            },
            SetRenderTreeParametersComponentsEnumStructTag::TypeButton => {
                Button::reconstruct_at(addr_of_mut!((*object).payload.payload_type_button) as *mut Button, references_pointer)
            },
        }
    }
}

pub struct SetRenderTreeParameters {
    pub components: Vec<SetRenderTreeParametersComponentsEnum>,
}

impl SetRenderTreeParameters {
    pub unsafe fn write_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        core::ptr::copy(self, pointer as *mut SetRenderTreeParameters, 1);
        pointer = pointer.offset(mem::size_of::<SetRenderTreeParameters>() as isize);

        mem::size_of::<SetRenderTreeParameters>() + self.write_references_at(pointer)
    }

    pub unsafe fn write_references_at(&self, pointer: *mut u8) -> usize {
        let mut pointer = pointer;
        let mut size: usize = 0;

        // OneOfType components
        let len = self.components.len();
        *(pointer as *mut usize) = len;
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        core::ptr::copy(self.components.as_ptr(), pointer as *mut SetRenderTreeParametersComponentsEnum, len);
        pointer = pointer.offset(len as isize * mem::size_of::<SetRenderTreeParametersComponentsEnum>() as isize);
        size += mem::size_of::<usize>() + len * mem::size_of::<SetRenderTreeParametersComponentsEnum>();
        for item in self.components.iter() {
            let item_size = item.write_references_at(pointer);
            pointer = pointer.offset(item_size as isize);
            size += item_size;
        }

        size
    }

    pub unsafe fn reconstruct_at_inline(object_pointer: *mut u8) -> usize {
        mem::size_of::<SetRenderTreeParameters>() + Self::reconstruct_at(object_pointer as *mut SetRenderTreeParameters, object_pointer.offset(mem::size_of::<SetRenderTreeParameters>() as isize))
    }

    pub unsafe fn reconstruct_at(object_pointer: *mut SetRenderTreeParameters, references_pointer: *mut u8) -> usize {
        let mut pointer = references_pointer;
        let mut size: usize = 0;

        // OneOfType components
        let len = *(pointer as *const usize);
        pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut assign = ManuallyDrop::new(Vec::from_raw_parts(pointer as *mut SetRenderTreeParametersComponentsEnum, len, len));
        core::ptr::write(addr_of_mut!((*object_pointer).components), ManuallyDrop::take(&mut assign));
        size += mem::size_of::<usize>() + len * mem::size_of::<SetRenderTreeParametersComponentsEnum>();
        let mut references_pointer = pointer.offset(len as isize * mem::size_of::<SetRenderTreeParametersComponentsEnum>() as isize);
        for item in (*object_pointer).components.iter() {
            let item_size = SetRenderTreeParametersComponentsEnum::reconstruct_at(pointer as *mut SetRenderTreeParametersComponentsEnum, references_pointer);
            pointer = pointer.offset(mem::size_of::<SetRenderTreeParametersComponentsEnum>() as isize);
            references_pointer = references_pointer.offset(item_size as isize);
            size += item_size;
        }
        pointer = references_pointer;

        size
    }
}



