#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

/// 定义erc20智能合约
#[ink::contract]
mod contracts_ink_erc20 {
    use ink_storage::collections::HashMap;

    // 定义存储
    #[ink(storage)]
    pub struct ContractsInkErc20 {
        total_supply: Balance,
        balances: HashMap<AccountId, Balance>,
        allowances: HashMap<(AccountId, AccountId), Balance>,
    }

    // 转移事件
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,

        #[ink(topic)]
        to: Option<AccountId>,

        #[ink(topic)]
        value: Balance,
    }

    // 授权某个账户指定额度事件
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        #[ink(topic)]
        value: Balance,
    }

    // 定义错误
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        InsufficientApproval,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl ContractsInkErc20 {
        // 构造器，指定初始化额度
        #[ink(constructor)]
        pub fn new(init_supply: Balance) -> Self {
            let caller = Self::env().caller();
            let mut balances = HashMap::new();
            balances.insert(caller, init_supply);

            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: init_supply,
            });

            Self {
                total_supply: init_supply,
                balances,
                allowances: HashMap::new(),
            }
        }

        // 总供应
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        // 账户余额
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_of_or_zero(&owner)
        }

        // 授权某个账户可操作额度
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), value);

            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });

            Ok(())
        }

        // 查询剩余可操作额度
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowance_of_or_zero(&owner, &spender)
        }

        // 从某个授权账户转移部分授权额度到指定账户
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance_of_or_zero(&from, &caller);
            if allowance < value {
                return Err(Error::InsufficientApproval);
            }

            self.transfer_from_to(from, to, value)?;

            self.allowances.insert((from, caller), allowance - value);
            Ok(())
        }

        // 转移部分资产到指定账户
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            self.transfer_from_to(self.env().caller(), to, value)
        }


        fn transfer_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let from_balance = self.balance_of_or_zero(&from);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }

            self.balances.insert(from, from_balance - value);
            let to_balance = self.balance_of_or_zero(&to);
            self.balances.insert(to, to_balance + value);

            Self::env().emit_event(Transfer {
                from: Some(from),
                to: Some(to),
                value,
            });

            Ok(())
        }

        fn balance_of_or_zero(&self, owner: &AccountId) -> Balance {
            *self.balances.get(owner).unwrap_or(&0)
        }

        fn allowance_of_or_zero(&self, owner: &AccountId, spender: &AccountId) -> Balance {
            *self.allowances.get(&(*owner, *spender)).unwrap_or(&0)
        }
    }

    // 单元测试
    #[cfg(test)]
    mod tests {
        use super::*;

        use ink_lang as ink;

        #[ink::test]
        fn new_works() {
            let contract = ContractsInkErc20::new(2022);
            assert_eq!(contract.total_supply(), 2022);
        }

        #[ink::test]
        fn balance_works() {
            let contract = ContractsInkErc20::new(100);
            assert_eq!(contract.total_supply(), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 0);
        }

        #[ink::test]
        fn transfer_works() {
            let mut contract = ContractsInkErc20::new(100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            assert_eq!(contract.transfer(AccountId::from([0x0; 32]), 10), Ok(()));
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 10);
            assert_eq!(
                contract.transfer(AccountId::from([0x0; 32]), 100),
                Err(Error::InsufficientBalance)
            );
        }

        #[ink::test]
        fn transfer_from_works() {
            let mut contract = ContractsInkErc20::new(100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            assert_eq!(contract.approve(AccountId::from([0x1; 32]), 20), Ok(()));
            assert_eq!(
                contract.transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 10),
                Ok(())
            );
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 10);
            assert_eq!(
                contract.transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 200),
                Err(Error::InsufficientApproval)
            );
        }

        #[ink::test]
        fn allowances_works() {
            let mut contract = ContractsInkErc20::new(100);
            assert_eq!(contract.balance_of(AccountId::from([0x1; 32])), 100);
            assert_eq!(contract.approve(AccountId::from([0x1; 32]), 200), Ok(()));
            assert_eq!(
                contract.allowance(AccountId::from([0x1; 32]), AccountId::from([0x1; 32])),
                200
            );

            assert_eq!(
                contract.transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 50),
                Ok(())
            );
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 50);
            assert_eq!(
                contract.allowance(AccountId::from([0x1; 32]), AccountId::from([0x1; 32])),
                150
            );

            assert_eq!(
                contract.transfer_from(AccountId::from([0x1; 32]), AccountId::from([0x0; 32]), 100),
                Err(Error::InsufficientBalance)
            );
            assert_eq!(contract.balance_of(AccountId::from([0x0; 32])), 50);
            assert_eq!(
                contract.allowance(AccountId::from([0x1; 32]), AccountId::from([0x1; 32])),
                150
            );
        }
    }
}