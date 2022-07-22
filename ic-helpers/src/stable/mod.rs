use stable_structures::{self, Memory, StableBTreeMap};

pub mod chunk_manager;

pub mod export {
    pub use stable_structures;
}

#[cfg(target_arch = "wasm32")]
pub type StableMemory = stable_structures::Ic0StableMemory;
#[cfg(not(target_arch = "wasm32"))]
pub type StableMemory = stable_structures::VectorMemory;

#[cfg(test)]
mod test {
    use super::{chunk_manager::VirtualMemory, export::stable_structures::RestrictedMemory, *};
    use std::rc::Rc;

    const WASM_PAGE_SIZE: u64 = 65536;

    #[test]
    fn single_entry_grow_size() {
        let manager_memory = StableMemory::default();
        let data_memory = StableMemory::default();
        let virtual_memory = VirtualMemory::init(data_memory, manager_memory, 0);

        assert_eq!(virtual_memory.size(), 0);
        assert_eq!(virtual_memory.grow(10), 0);
        assert_eq!(virtual_memory.size(), 10);

        // The layout should look like this:
        //
        // manager_memory:
        // ------------------------------------------------------------------------- <- Address 0
        //      StableBTreeMap<vec![0, 0, 0, 0], vec![]>
        //                          ↕   \   /                                             \
        //   virtual_memory index is 0; data_memory page 0 belongs to virtual_memory;
        //                                                                                  \
        //      StableBTreeMap<vec![0, 0, 0, 1], vec![]>
        //                          ↕   \   /                                           manager_memory usable page 0
        //   virtual_memory index is 0; data_memory page 1 belongs to virtual_memory;
        // ...                                                                              /
        //      StableBTreeMap<vec![0, 0, 0, 9], vec![]>
        //                          ↕   \   /                                             /
        //   virtual_memory index is 0; data_memory page 9 belongs to virtual_memory;
        // ...
        // ------------------------------------------------------------------------- <- Address 65536
        //                                                                              manager_memory potential pages
        //
        //
        // data_memory:
        // ------------------------------------------------------------------------- <- Address 0
        //      vec![0; 10 * WASM_PAGE_SIZE as usize]                                   data_memory usable pages [0, 10)
        //                                                                              virtual_memory usable pages [0, 10)
        //
        // ------------------------------------------------------------------------- <- Address 65536 * 10
        //                                                                              data_memory potential pages
        //
    }

    #[test]
    fn multiple_entry_grow_size() {
        let manager_memory = StableMemory::default();
        let data_memory = StableMemory::default();

        let virtual_memory_0 =
            VirtualMemory::init(Rc::clone(&data_memory), Rc::clone(&manager_memory), 0);
        let virtual_memory_1 =
            VirtualMemory::init(Rc::clone(&data_memory), Rc::clone(&manager_memory), 1);

        assert_eq!(virtual_memory_0.size(), 0);
        assert_eq!(virtual_memory_1.size(), 0);

        assert_eq!(virtual_memory_0.grow(5), 0);
        virtual_memory_1.page_range.borrow_mut().reload();
        assert_eq!(virtual_memory_1.grow(6), 0);

        assert_eq!(virtual_memory_0.grow(7), 5);
        virtual_memory_1.page_range.borrow_mut().reload();
        assert_eq!(virtual_memory_1.grow(8), 6);

        assert_eq!(virtual_memory_0.size(), 12);
        assert_eq!(virtual_memory_1.size(), 14);

        // The layout should look like this:
        //
        // manager_memory:
        // ------------------------------------------------------------------------- <- Address 0
        //      StableBTreeMap<vec![0, 0, 0, 0], vec![]>
        //                          ↕   \   /                                             \
        //   virtual_memory index is 0; data_memory page 0 belongs to virtual_memory_0;
        // ...                                                                             \
        //      StableBTreeMap<vec![0, 0, 0, 4], vec![]>
        //                          ↕   \   /                                               \
        //   virtual_memory index is 0; data_memory page 4 belongs to virtual_memory_0;
        //
        //      StableBTreeMap<vec![0, 0, 0, 11], vec![]>                                    \
        //                          ↕   \   /
        //   virtual_memory index is 0; data_memory page 11 belongs to virtual_memory_0;      \
        // ...
        //      StableBTreeMap<vec![0, 0, 0, 17], vec![]>                                      \
        //                          ↕   \   /
        //   virtual_memory index is 0; data_memory page 17 belongs to virtual_memory_0;
        //                                                                                 manager_memory usable page 0
        //      StableBTreeMap<vec![1, 0, 0, 5], vec![]>
        //                          ↕   \   /
        //   virtual_memory index is 1; data_memory page 5 belongs to virtual_memory_1;         /
        // ...
        //      StableBTreeMap<vec![1, 0, 0, 10], vec![]>                                      /
        //                          ↕   \   /
        //   virtual_memory index is 1; data_memory page 10 belongs to virtual_memory_1;      /
        //
        //      StableBTreeMap<vec![1, 0, 0, 18], vec![]>
        //                          ↕   \   /                                                /
        //   virtual_memory index is 1; data_memory page 18 belongs to virtual_memory_1;
        // ...                                                                              /
        //      StableBTreeMap<vec![1, 0, 0, 25], vec![]>
        //                          ↕   \   /                                              /
        //   virtual_memory index is 1; data_memory page 15 belongs to virtual_memory_1;
        // ...
        // ------------------------------------------------------------------------- <- Address 65536
        //                                                                              manager_memory potential pages
        //
        //
        // data_memory:
        // ------------------------------------------------------------------------- <- Address 0
        //      vec![0; 5 * WASM_PAGE_SIZE as usize]                                    data_memory usable pages [0, 5)
        //                                                                              virtual_memory_0 usable pages [0, 5)
        //
        // ------------------------------------------------------------------------- <- Address 65536 * 5
        //      vec![0; 6 * WASM_PAGE_SIZE as usize]                                    data_memory usable pages [5, 11)
        //                                                                              virtual_memory_1 usable pages [0, 6)
        //
        // ------------------------------------------------------------------------- <- Address 65536 * 11
        //      vec![0; 7 * WASM_PAGE_SIZE as usize]                                    data_memory usable pages [11, 18),
        //                                                                              virtual_memory_0 usable pages [5, 12)
        //
        // ------------------------------------------------------------------------- <- Address 65536 * 18
        //      vec![0; 8 * WASM_PAGE_SIZE as usize]                                    data_memory usable pages [18, 26)
        //                                                                              virtual_memory_1 usable pages [6, 14)
        //
        // ------------------------------------------------------------------------- <- Address 65536 * 26
        //                                                                              data_memory potential pages
        //
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn write_without_enough_space() {
        let manager_memory = StableMemory::default();
        let data_memory = StableMemory::default();
        let virtual_memory = VirtualMemory::init(data_memory, manager_memory, 0);

        assert_eq!(virtual_memory.grow(1), 0);
        let src = [1; 1 + WASM_PAGE_SIZE as usize];
        virtual_memory.write(0, &src);

        // The layout should look like this:
        //
        // manager_memory:
        // ------------------------------------------------------------------------- <- Address 0
        //      StableBTreeMap<vec![0, 0, 0, 0], vec![]>                                manager_memory usable page 0
        //                          ↕   \   /
        //   virtual_memory index is 0; data_memory page 0 belongs to virtual_memory;
        // ...
        // ------------------------------------------------------------------------- <- Address 65536
        //                                                                              manager_memory potential pages
        //
        //
        // data_memory:
        // ------------------------------------------------------------------------- <- Address 0
        //      vec![0; WASM_PAGE_SIZE as usize]                                        data_memory usable page 0
        //                                                                              virtual_memory usable page 0
        //
        // ------------------------------------------------------------------------- <- Address 65536 -> belongs to potential pages, virtual_memory try to write to it, panic.
        //
    }

    #[test]
    #[should_panic(expected = "grow failed, which return -1")]
    fn write_without_grow_further() {
        let manager_memory = StableMemory::default();
        let data_memory = RestrictedMemory::new(StableMemory::default(), 0..1);
        let virtual_memory = VirtualMemory::init(data_memory, manager_memory, 0);

        let result = virtual_memory.grow(2);
        assert_eq!(result, 0, "grow failed, which return {}", result);
        let src = [1; 1 + WASM_PAGE_SIZE as usize];
        virtual_memory.write(0, &src);

        // The layout should look like this:
        //
        // manager_memory:
        // ------------------------------------------------------------------------- <- Address 0
        //                                                                              manager_memory potential pages;
        //
        //
        // data_memory:
        // ------------------------------------------------------------------------- <- Address 0
        //                                                                              data_memory only 1 potential page;
        //
        // ------------------------------------------------------------------------- <- Address 65536
        //
        // The data_memory's capacity is only 1 page, but virtual_memory try to grow 2 pages. Panic and state changes will be rolled back.
    }

    #[test]
    fn write_multiple_data_to_same_page() {
        let manager_memory = StableMemory::default();
        let data_memory = StableMemory::default();

        let virtual_memory_0 =
            VirtualMemory::init(Rc::clone(&data_memory), Rc::clone(&manager_memory), 0);
        let virtual_memory_1 =
            VirtualMemory::init(Rc::clone(&data_memory), Rc::clone(&manager_memory), 0);

        assert_eq!(virtual_memory_0.grow(1), 0);
        virtual_memory_1.page_range.borrow_mut().reload();

        let src_0 = [1; WASM_PAGE_SIZE as usize];
        let mut dst_0 = [0; WASM_PAGE_SIZE as usize];
        virtual_memory_0.write(0, &src_0);
        virtual_memory_0.read(0, &mut dst_0);
        assert_eq!(src_0, dst_0);

        let src_1 = [2; WASM_PAGE_SIZE as usize];
        let mut dst_1 = [0; WASM_PAGE_SIZE as usize];
        virtual_memory_1.read(0, &mut dst_1);
        assert_eq!(dst_1, src_0);

        virtual_memory_1.write(0, &src_1);
        virtual_memory_1.read(0, &mut dst_1);
        assert_eq!(dst_1, src_1);

        // The layout should look like this:
        //
        // manager_memory:
        // ------------------------------------------------------------------------- <- Address 0
        //      StableBTreeMap<vec![0, 0, 0, 0], vec![]>                                manager_memory usable page 0
        //                          ↕   \   /
        //   virtual_memory index is 0; data_memory page 0 belongs to virtual_memory_0 & virtual_memory_1;
        // ...
        // ------------------------------------------------------------------------- <- Address 65536
        //                                                                              manager_memory potential pages
        //
        //
        // data_memory:
        // ------------------------------------------------------------------------- <- Address 0
        //      vec![2; WASM_PAGE_SIZE as usize]                                        data_memory usable page 0
        //                                                                              virtual_memory_0 usable page 0
        //                                                                              virtual_memory_1 usable page 0
        //
        // ------------------------------------------------------------------------- <- Address 65536
        //                                                                              data_memory potential pages;
        //
        // Because virtual_memory_0 and virtual_memory_1 use the same virtual_memory index 0, they will all have the same memory pages.
    }

    #[test]
    fn write_single_entry_spanning_multiple_pages() {
        let manager_memory = StableMemory::default();
        let data_memory = StableMemory::default();
        let virtual_memory = VirtualMemory::init(data_memory, manager_memory, 0);

        assert_eq!(virtual_memory.grow(3), 0);
        let src = [1; 1 + 2 * WASM_PAGE_SIZE as usize];
        let mut dst = [0; 1 + 2 * WASM_PAGE_SIZE as usize];
        virtual_memory.write(0, &src);
        virtual_memory.read(0, &mut dst);
        assert_eq!(src, dst);

        // The layout should look like this:
        //
        // manager_memory:
        // ------------------------------------------------------------------------- <- Address 0
        //      StableBTreeMap<vec![0, 0, 0, 0], vec![]>
        //                          ↕   \   /                                             \
        //   virtual_memory index is 0; data_memory page 0 belongs to virtual_memory;
        //                                                                                  \
        //      StableBTreeMap<vec![0, 0, 0, 1], vec![]>
        //                          ↕   \   /                                           manager_memory usable page 0
        //   virtual_memory index is 0; data_memory page 1 belongs to virtual_memory;
        //                                                                                  /
        //      StableBTreeMap<vec![0, 0, 0, 2], vec![]>
        //                          ↕   \   /                                             /
        //   virtual_memory index is 0; data_memory page 2 belongs to virtual_memory;
        // ...
        // ------------------------------------------------------------------------- <- Address 65536
        //                                                                              manager_memory potential pages
        //
        //
        // data_memory:
        // ------------------------------------------------------------------------- <- Address 0
        //      vec![1; WASM_PAGE_SIZE as usize]                                        data_memory usable page 0
        //                                                                              virtual_memory usable page 0
        //
        // ------------------------------------------------------------------------- <- Address 65536
        //      vec![1; WASM_PAGE_SIZE as usize]                                        data_memory usable page 1
        //                                                                              virtual_memory usable page 1
        //
        // ------------------------------------------------------------------------- <- Address 65536 * 2
        //      vec![1]                                                                 data_memory usable page 2
        //      vec![0; WASM_PAGE_SIZE as usize - 1 ]                                   virtual_memory usable page 2
        //
        // ------------------------------------------------------------------------- <- Address 65536 * 3
        //                                                                              data_memory potential pages;
        //
    }

    #[test]
    fn write_multiple_entries_spanning_multiple_pages() {
        let manager_memory = StableMemory::default();
        let data_memory = StableMemory::default();

        let virtual_memory_0 =
            VirtualMemory::init(Rc::clone(&data_memory), Rc::clone(&manager_memory), 0);
        let virtual_memory_1 =
            VirtualMemory::init(Rc::clone(&data_memory), Rc::clone(&manager_memory), 1);
        let virtual_memory_2 =
            VirtualMemory::init(Rc::clone(&data_memory), Rc::clone(&manager_memory), 2);

        assert_eq!(virtual_memory_0.grow(1), 0);
        virtual_memory_1.page_range.borrow_mut().reload();
        assert_eq!(virtual_memory_1.grow(1), 0);
        virtual_memory_2.page_range.borrow_mut().reload();
        assert_eq!(virtual_memory_2.grow(1), 0);

        assert_eq!(virtual_memory_0.grow(1), 1);
        assert_eq!(virtual_memory_1.grow(1), 1);
        assert_eq!(virtual_memory_2.grow(1), 1);

        assert_eq!(virtual_memory_0.grow(1), 2);
        assert_eq!(virtual_memory_1.grow(1), 2);
        assert_eq!(virtual_memory_2.grow(1), 2);

        let src_0 = [1; 3 * WASM_PAGE_SIZE as usize - 2];
        let src_1 = [2; 3 * WASM_PAGE_SIZE as usize - 2];
        let src_2 = [3; 3 * WASM_PAGE_SIZE as usize - 2];

        virtual_memory_0.write(1, &src_0);
        virtual_memory_1.write(1, &src_1);
        virtual_memory_2.write(1, &src_2);

        let mut dst_0 = [0; 3 * WASM_PAGE_SIZE as usize - 2];
        let mut dst_1 = [0; 3 * WASM_PAGE_SIZE as usize - 2];
        let mut dst_2 = [0; 3 * WASM_PAGE_SIZE as usize - 2];

        virtual_memory_0.read(1, &mut dst_0);
        virtual_memory_1.read(1, &mut dst_1);
        virtual_memory_2.read(1, &mut dst_2);

        assert_eq!(src_0, dst_0);
        assert_eq!(src_1, dst_1);
        assert_eq!(src_2, dst_2);

        // The layout should look like this:
        //
        // manager_memory:
        // ------------------------------------------------------------------------- <- Address 0
        //      StableBTreeMap<vec![0, 0, 0, 0], vec![]>
        //                          ↕   \   /                                             \
        //   virtual_memory index is 0; data_memory page 0 belongs to virtual_memory_0;
        //                                                                                 \
        //      StableBTreeMap<vec![0, 0, 0, 3], vec![]>
        //                          ↕   \   /                                               \
        //   virtual_memory index is 0; data_memory page 3 belongs to virtual_memory_0;
        //
        //      StableBTreeMap<vec![0, 0, 0, 6], vec![]>                                     \
        //                          ↕   \   /
        //   virtual_memory index is 0; data_memory page 6 belongs to virtual_memory_0;       \
        //
        //      StableBTreeMap<vec![1, 0, 0, 1], vec![]>                                       \
        //                          ↕   \   /
        //   virtual_memory index is 1; data_memory page 1 belongs to virtual_memory_1;
        //                                                                                   manager_memory usable page 0
        //      StableBTreeMap<vec![1, 0, 0, 4], vec![]>
        //                          ↕   \   /
        //   virtual_memory index is 1; data_memory page 4 belongs to virtual_memory_1;        /
        //
        //      StableBTreeMap<vec![1, 0, 0, 7], vec![]>                                      /
        //                          ↕   \   /
        //   virtual_memory index is 1; data_memory page 7 belongs to virtual_memory_1;      /
        //
        //      StableBTreeMap<vec![2, 0, 0, 2], vec![]>
        //                          ↕   \   /                                               /
        //   virtual_memory index is 2; data_memory page 2 belongs to virtual_memory_2;
        //                                                                                 /
        //      StableBTreeMap<vec![2, 0, 0, 5], vec![]>
        //                          ↕   \   /                                             /
        //   virtual_memory index is 2; data_memory page 5 belongs to virtual_memory_2;
        //
        //      StableBTreeMap<vec![2, 0, 0, 8], vec![]>                                 /
        //                          ↕   \   /
        //   virtual_memory index is 2; data_memory page 8 belongs to virtual_memory_2;
        // ...
        // ------------------------------------------------------------------------- <- Address 65536
        //                                                                              manager_memory potential pages
        //
        //
        // data_memory:
        // ------------------------------------------------------------------------- <- Address 0
        //      vec![0]                                                                 data_memory usable page 0
        //      vec![1; WASM_PAGE_SIZE as usize - 1]                                    virtual_memory_0 page 0
        //
        // ------------------------------------------------------------------------- <- Address 65536
        //      vec![0]                                                                 data_memory usable page 1
        //      vec![2; WASM_PAGE_SIZE as usize - 1]                                    virtual_memory_1 page 0
        //
        // ------------------------------------------------------------------------- <- Address 65536 * 2
        //      vec![0]                                                                 data_memory usable page 2
        //      vec![3; WASM_PAGE_SIZE as usize - 1]                                    virtual_memory_2 page 0
        //
        // ------------------------------------------------------------------------- <- Address 65536 * 3
        //      vec![1; WASM_PAGE_SIZE as usize]                                        data_memory usable page 3
        //                                                                              virtual_memory_0 page 1
        //
        // ------------------------------------------------------------------------- <- Address 65536 * 4
        //      vec![2; WASM_PAGE_SIZE as usize]                                        data_memory usable page 4
        //                                                                              virtual_memory_1 page 1
        //
        // ------------------------------------------------------------------------- <- Address 65536 * 5
        //      vec![3; WASM_PAGE_SIZE as usize]                                        data_memory usable page 5
        //                                                                              virtual_memory_2 page 1
        //
        // ------------------------------------------------------------------------- <- Address 65536 * 6
        //      vec![1; WASM_PAGE_SIZE as usize - 1]                                    data_memory usable page 6
        //      vec![0]                                                                 virtual_memory_0 page 2
        //
        // ------------------------------------------------------------------------- <- Address 65536 * 7
        //      vec![2; WASM_PAGE_SIZE as usize - 1]                                    data_memory usable page 7
        //      vec![0]                                                                 virtual_memory_1 page 2
        //
        // ------------------------------------------------------------------------- <- Address 65536 * 8
        //      vec![3; WASM_PAGE_SIZE as usize - 1]                                    data_memory usable page 8
        //      vec![0]                                                                 virtual_memory_2 page 2
        //
        // ------------------------------------------------------------------------- <- Address 65536 * 9
        //                                                                              data_memory potential pages;
        //
    }

    #[test]
    #[should_panic(expected = "out of bounds")]
    fn read_outside_of_memory_range() {
        let manager_memory = StableMemory::default();
        let data_memory = StableMemory::default();
        let virtual_memory = VirtualMemory::init(data_memory, manager_memory, 0);

        let mut dst = [1; WASM_PAGE_SIZE as usize];
        virtual_memory.write(0, &mut dst);
    }

    #[test]
    fn read_single_entry_across_multiple_pages() {
        let manager_memory = StableMemory::default();
        let data_memory = StableMemory::default();
        let virtual_memory = VirtualMemory::init(data_memory, manager_memory, 0);

        assert_eq!(virtual_memory.grow(3), 0);
        let src = [1; 3 * WASM_PAGE_SIZE as usize];
        virtual_memory.write(0, &src);

        let mut dst = [0; 3 * WASM_PAGE_SIZE as usize];
        virtual_memory.read(0, &mut dst);
        assert_eq!(src, dst);
    }

    #[test]
    fn read_multiple_entries_across_multiple_pages() {
        let manager_memory = StableMemory::default();
        let data_memory = StableMemory::default();
        let virtual_memory_0 =
            VirtualMemory::init(Rc::clone(&data_memory), Rc::clone(&manager_memory), 0);
        let virtual_memory_1 =
            VirtualMemory::init(Rc::clone(&data_memory), Rc::clone(&manager_memory), 1);

        assert_eq!(virtual_memory_0.grow(1), 0);
        virtual_memory_1.page_range.borrow_mut().reload();
        assert_eq!(virtual_memory_1.grow(1), 0);

        assert_eq!(virtual_memory_0.grow(1), 1);
        assert_eq!(virtual_memory_1.grow(1), 1);

        let mut dst_0 = [1; 2 * WASM_PAGE_SIZE as usize];
        let mut dst_1 = [1; 2 * WASM_PAGE_SIZE as usize];

        virtual_memory_0.read(0, &mut dst_0);
        virtual_memory_0.read(0, &mut dst_1);

        assert_eq!(dst_0, [0; 2 * WASM_PAGE_SIZE as usize]);
        assert_eq!(dst_1, [0; 2 * WASM_PAGE_SIZE as usize]);
    }
}
