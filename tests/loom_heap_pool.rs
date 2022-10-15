#![cfg(loom)]

#[macro_use]
mod loom_helpers;

use self::loom_helpers::*;
use drone_core::heap::Pool;

const SIZE: isize = 16;

fn make_pool(count: usize, freed: usize) -> (isize, &'static Pool) {
    let memory = Box::leak(vec![0_u8; SIZE as usize * count].into_boxed_slice());
    let addr = memory.as_mut_ptr() as isize;
    let pool: &'static _ = Box::leak(Box::new(Pool::new(addr as usize, SIZE as usize, count)));
    let allocated = (0..freed).map(|_| pool.allocate().unwrap()).collect::<Vec<_>>();
    allocated.into_iter().for_each(|ptr| unsafe { pool.deallocate(ptr) });
    (addr, pool)
}

macro_rules! join_allocate {
    ($($x:ident),+$(,)?) => {{
        $(
            let $x = $x.join().unwrap().map_or(-1, |ptr| ptr.as_ptr() as isize);
        )*
        ($($x,)*)
    }};
}

#[test]
fn loom_allocate_allocate() {
    loom::model(|| {
        let (_addr, pool) = make_pool(0, 0);
        let x = loom::thread::spawn(move || pool.allocate());
        let y = loom::thread::spawn(move || pool.allocate());
        let (x, y) = join_allocate!(x, y);
        assert!(pool.allocate().is_none());
        assert!(x == -1 && y == -1);
    });
    let states = statemap![0 => [1, 2]];
    loom::model(|| {
        let (addr, pool) = make_pool(1, 0);
        let x = loom::thread::spawn(move || pool.allocate());
        let y = loom::thread::spawn(move || pool.allocate());
        let (x, y) = join_allocate!(x, y);
        assert!(pool.allocate().is_none());
        statemap_put(states, 0, match (x, y) {
            (x, -1) if x == addr => 1,
            (-1, y) if y == addr => 2,
            _ => 3,
        });
    });
    statemap_check_exhaustive(states);
    let states = statemap![0 => [1, 2]];
    loom::model(|| {
        let (addr, pool) = make_pool(1, 1);
        let x = loom::thread::spawn(move || pool.allocate());
        let y = loom::thread::spawn(move || pool.allocate());
        let (x, y) = join_allocate!(x, y);
        assert!(pool.allocate().is_none());
        statemap_put(states, 0, match (x, y) {
            (x, -1) if x == addr => 1,
            (-1, y) if y == addr => 2,
            _ => 3,
        });
    });
    statemap_check_exhaustive(states);
    let states = statemap![0 => [1, 2]];
    loom::model(|| {
        let (addr, pool) = make_pool(2, 0);
        let x = loom::thread::spawn(move || pool.allocate());
        let y = loom::thread::spawn(move || pool.allocate());
        let (x, y) = join_allocate!(x, y);
        assert!(pool.allocate().is_none());
        statemap_put(states, 0, match (x, y) {
            (x, y) if x == addr && y == addr + SIZE => 1,
            (x, y) if x == addr + SIZE && y == addr => 2,
            _ => 3,
        });
    });
    statemap_check_exhaustive(states);
    let states = statemap![0 => [1, 2]];
    loom::model(|| {
        let (addr, pool) = make_pool(2, 1);
        let x = loom::thread::spawn(move || pool.allocate());
        let y = loom::thread::spawn(move || pool.allocate());
        let (x, y) = join_allocate!(x, y);
        assert!(pool.allocate().is_none());
        statemap_put(states, 0, match (x, y) {
            (x, y) if x == addr && y == addr + SIZE => 1,
            (x, y) if x == addr + SIZE && y == addr => 2,
            _ => 3,
        });
    });
    statemap_check_exhaustive(states);
    let states = statemap![0 => [1, 2]];
    loom::model(|| {
        let (addr, pool) = make_pool(2, 2);
        let x = loom::thread::spawn(move || pool.allocate());
        let y = loom::thread::spawn(move || pool.allocate());
        let (x, y) = join_allocate!(x, y);
        assert!(pool.allocate().is_none());
        statemap_put(states, 0, match (x, y) {
            (x, y) if x == addr && y == addr + SIZE => 1,
            (x, y) if x == addr + SIZE && y == addr => 2,
            _ => 3,
        });
    });
    statemap_check_exhaustive(states);
}

#[test]
fn loom_allocate_deallocate() {
    let states = statemap![0 => [1, 2]];
    loom::model(|| {
        let (addr, pool) = make_pool(1, 0);
        let a = pool.allocate().unwrap();
        let x = loom::thread::spawn(move || unsafe { pool.deallocate(a) });
        let y = loom::thread::spawn(move || pool.allocate());
        x.join().unwrap();
        let (y,) = join_allocate!(y,);
        statemap_put(states, 0, match y {
            y if y == addr => {
                assert!(pool.allocate().is_none());
                1
            }
            -1 => {
                assert_eq!(pool.allocate().unwrap().as_ptr() as isize, addr);
                assert!(pool.allocate().is_none());
                2
            }
            _ => 3,
        });
    });
    statemap_check_exhaustive(states);
    let states = statemap![0 => [1, 2]];
    loom::model(|| {
        let (addr, pool) = make_pool(2, 0);
        let a = pool.allocate().unwrap();
        let x = loom::thread::spawn(move || unsafe { pool.deallocate(a) });
        let y = loom::thread::spawn(move || pool.allocate());
        x.join().unwrap();
        let (y,) = join_allocate!(y,);
        statemap_put(states, 0, match y {
            y if y == addr => {
                assert_eq!(pool.allocate().unwrap().as_ptr() as isize, addr + SIZE);
                assert!(pool.allocate().is_none());
                1
            }
            y if y == addr + SIZE => {
                assert_eq!(pool.allocate().unwrap().as_ptr() as isize, addr);
                assert!(pool.allocate().is_none());
                2
            }
            _ => 3,
        });
    });
    statemap_check_exhaustive(states);
    let states = statemap![0 => [1, 2]];
    loom::model(|| {
        let (addr, pool) = make_pool(2, 1);
        let a = pool.allocate().unwrap();
        let x = loom::thread::spawn(move || unsafe { pool.deallocate(a) });
        let y = loom::thread::spawn(move || pool.allocate());
        x.join().unwrap();
        let (y,) = join_allocate!(y,);
        statemap_put(states, 0, match y {
            y if y == addr => {
                assert_eq!(pool.allocate().unwrap().as_ptr() as isize, addr + SIZE);
                assert!(pool.allocate().is_none());
                1
            }
            y if y == addr + SIZE => {
                assert_eq!(pool.allocate().unwrap().as_ptr() as isize, addr);
                assert!(pool.allocate().is_none());
                2
            }
            _ => 3,
        });
    });
    statemap_check_exhaustive(states);
}

#[test]
fn loom_deallocate_deallocate() {
    let states = statemap![0 => [1, 2]];
    loom::model(|| {
        let (addr, pool) = make_pool(2, 0);
        let x = pool.allocate().unwrap();
        let y = pool.allocate().unwrap();
        let x = loom::thread::spawn(move || unsafe { pool.deallocate(x) });
        let y = loom::thread::spawn(move || unsafe { pool.deallocate(y) });
        x.join().unwrap();
        y.join().unwrap();
        let x = pool.allocate().unwrap().as_ptr() as isize;
        let y = pool.allocate().unwrap().as_ptr() as isize;
        assert!(pool.allocate().is_none());
        statemap_put(states, 0, match (x, y) {
            (x, y) if x == addr && y == addr + SIZE => 1,
            (x, y) if x == addr + SIZE && y == addr => 2,
            _ => 3,
        });
    });
    statemap_check_exhaustive(states);
}
