#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::mem;
use core::mem::ManuallyDrop;
use crate::types::*;
use crate::enums::*;

pub enum SetRenderTreeParametersComponentsEnum {
    TypeWindow(*mut Window),
    TypeVerticalLayout(*mut VerticalLayout),
    TypeHorizontalLayout(*mut HorizontalLayout),
    TypeGridLayout(*mut GridLayout),
    TypeGridLayoutColumn(*mut GridLayoutColumn),
    TypeGridLayoutRow(*mut GridLayoutRow),
    TypeButton(*mut Button),
}

impl SetRenderTreeParametersComponentsEnum {
    pub const OPTION_WINDOW: usize = 1;
    pub const OPTION_VERTICALLAYOUT: usize = 2;
    pub const OPTION_HORIZONTALLAYOUT: usize = 3;
    pub const OPTION_GRIDLAYOUT: usize = 4;
    pub const OPTION_GRIDLAYOUTCOLUMN: usize = 5;
    pub const OPTION_GRIDLAYOUTROW: usize = 6;
    pub const OPTION_BUTTON: usize = 7;

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        let base_pointer = pointer;
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let size: usize = mem::size_of::<usize>();

        let size = match self {
            SetRenderTreeParametersComponentsEnum::TypeWindow(value) => {
                *(base_pointer as *mut usize) = Self::OPTION_WINDOW;
                (value.as_ref().unwrap()).write_at_address(pointer)
            },
            SetRenderTreeParametersComponentsEnum::TypeVerticalLayout(value) => {
                *(base_pointer as *mut usize) = Self::OPTION_VERTICALLAYOUT;
                (value.as_ref().unwrap()).write_at_address(pointer)
            },
            SetRenderTreeParametersComponentsEnum::TypeHorizontalLayout(value) => {
                *(base_pointer as *mut usize) = Self::OPTION_HORIZONTALLAYOUT;
                (value.as_ref().unwrap()).write_at_address(pointer)
            },
            SetRenderTreeParametersComponentsEnum::TypeGridLayout(value) => {
                *(base_pointer as *mut usize) = Self::OPTION_GRIDLAYOUT;
                (value.as_ref().unwrap()).write_at_address(pointer)
            },
            SetRenderTreeParametersComponentsEnum::TypeGridLayoutColumn(value) => {
                *(base_pointer as *mut usize) = Self::OPTION_GRIDLAYOUTCOLUMN;
                (value.as_ref().unwrap()).write_at_address(pointer)
            },
            SetRenderTreeParametersComponentsEnum::TypeGridLayoutRow(value) => {
                *(base_pointer as *mut usize) = Self::OPTION_GRIDLAYOUTROW;
                (value.as_ref().unwrap()).write_at_address(pointer)
            },
            SetRenderTreeParametersComponentsEnum::TypeButton(value) => {
                *(base_pointer as *mut usize) = Self::OPTION_BUTTON;
                (value.as_ref().unwrap()).write_at_address(pointer)
            },
        };

        mem::size_of::<usize>() + size
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, Self) {
        let enum_type = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);

        let (size, object) = match enum_type {
            Self::OPTION_WINDOW => {
                let (size, value) = Window::get_from_address(pointer);
                (size, Self::TypeWindow(value))
            }
            Self::OPTION_VERTICALLAYOUT => {
                let (size, value) = VerticalLayout::get_from_address(pointer);
                (size, Self::TypeVerticalLayout(value))
            }
            Self::OPTION_HORIZONTALLAYOUT => {
                let (size, value) = HorizontalLayout::get_from_address(pointer);
                (size, Self::TypeHorizontalLayout(value))
            }
            Self::OPTION_GRIDLAYOUT => {
                let (size, value) = GridLayout::get_from_address(pointer);
                (size, Self::TypeGridLayout(value))
            }
            Self::OPTION_GRIDLAYOUTCOLUMN => {
                let (size, value) = GridLayoutColumn::get_from_address(pointer);
                (size, Self::TypeGridLayoutColumn(value))
            }
            Self::OPTION_GRIDLAYOUTROW => {
                let (size, value) = GridLayoutRow::get_from_address(pointer);
                (size, Self::TypeGridLayoutRow(value))
            }
            Self::OPTION_BUTTON => {
                let (size, value) = Button::get_from_address(pointer);
                (size, Self::TypeButton(value))
            }
            _ => {
                panic!("Unknown enum type");
            }
        };

        (mem::size_of::<usize>() + size, object)
    }
}

pub struct SetRenderTreeParameters {
    pub components: Vec<SetRenderTreeParametersComponentsEnum>,
}

impl SetRenderTreeParameters {
    pub unsafe fn create_at_address(pointer: *mut u8, components: Vec<SetRenderTreeParametersComponentsEnum>) -> usize {
        let object: *mut SetRenderTreeParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<SetRenderTreeParameters>() as isize);

        // components
        *(pointer as *mut usize) = components.len();
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _components_size: usize = mem::size_of::<usize>();
        for item in components.iter() {
            let item_size = item.write_at_address(pointer);
            let pointer = pointer.offset(item_size as isize);
            _components_size += item_size;
        }

        // return
        mem::size_of::<SetRenderTreeParameters>() + _components_size
    }

    pub unsafe fn write_at_address(&self, pointer: *mut u8) -> usize {
        core::ptr::copy(self, pointer as *mut SetRenderTreeParameters, 1);
        let pointer = pointer.offset(mem::size_of::<SetRenderTreeParameters>() as isize);

        // components
        *(pointer as *mut usize) = self.components.len();
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _components_size: usize = mem::size_of::<usize>();
        for item in self.components.iter() {
            let item_size = item.write_at_address(pointer);
            let pointer = pointer.offset(item_size as isize);
            _components_size += item_size;
        }

        // return
        mem::size_of::<SetRenderTreeParameters>() + _components_size
    }

    pub unsafe fn get_from_address(pointer: *mut u8) -> (usize, *mut Self) {
        let object: *mut SetRenderTreeParameters = mem::transmute(pointer);
        let pointer = pointer.offset(mem::size_of::<SetRenderTreeParameters>() as isize);

        // components
        let components_count = *(pointer as *mut usize);
        let pointer = pointer.offset(mem::size_of::<usize>() as isize);
        let mut _components_size: usize = mem::size_of::<usize>();
        let mut _components_vec: Vec<SetRenderTreeParametersComponentsEnum> = Vec::with_capacity(_components_size);
        for _ in 0..components_count {
            let (item_size, item) = SetRenderTreeParametersComponentsEnum::get_from_address(pointer);
            _components_vec.push(item);
            let pointer = pointer.offset(item_size as isize);
            _components_size += item_size;
        }
        (*object).components = _components_vec;

        // return
        (mem::size_of::<SetRenderTreeParameters>() + _components_size, object)
    }
}


