#[starknet::contract]
pub mod sn_c1 {
    #[storage]
    struct Storage {}
}

#[derive(Introspect, Drop, Serde)]
#[dojo::model]
pub struct M {
    #[key]
    pub a: felt252,
    pub b: felt252,
}

#[derive(Introspect, Drop, Serde)]
#[dojo::model]
pub struct M2 {
    #[key]
    pub a: u32,
    pub b: u256,
}

#[dojo::interface]
pub trait MyInterface {
    fn system_1(ref world: IWorldDispatcher, data: felt252) -> felt252;
    fn system_2(ref world: IWorldDispatcher);
    fn view_1(world: @IWorldDispatcher) -> felt252;
}

#[dojo::contract]
pub mod c1 {
    use super::MyInterface;

    fn dojo_init(self: @ContractState, arg1: felt252) {
        let _arg1 = arg1;
    }

    #[abi(embed_v0)]
    impl MyInterfaceImpl of MyInterface<ContractState> {
        fn system_1(ref world: IWorldDispatcher, data: felt252) -> felt252 {
            let _world = world;
            55
        }

        fn system_2(ref world: IWorldDispatcher) {
            let _world = world;
        }

        fn view_1(world: @IWorldDispatcher) -> felt252 {
            let _world = world;
            89
        }
    }
}

#[dojo::contract]
pub mod c2 {}
