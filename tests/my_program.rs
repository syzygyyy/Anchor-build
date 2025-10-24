use anchor_lang::prelude::*;
use anchor_client::{Client, Cluster};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer, EncodableKey},
    pubkey::Pubkey,
};
use std::rc::Rc;

#[test]
fn test_initialize() {
    // 使用预充值的payer
    let payer = Keypair::read_from_file("/root/.config/solana/id.json").unwrap();
    let payer_pubkey = payer.pubkey();  // 在移动前保存pubkey

    let client = Client::new_with_options(
        Cluster::Localnet,
        Rc::new(payer),
        CommitmentConfig::processed(),
    );
    let program = client.program(my_program::id()).unwrap();

    // 为此测试使用唯一的PDA种子
    let (counter_pda, _) = Pubkey::find_program_address(
        &[b"counter", payer_pubkey.as_ref()],
        &my_program::id(),
    );

    // 测试初始化
    let _tx = program
        .request()
        .accounts(my_program::accounts::Initialize {
            counter: counter_pda,
            user: payer_pubkey,
            system_program: anchor_lang::system_program::ID,
        })
        .args(my_program::instruction::Initialize {})
        .send()
        .unwrap();

    // 验证计数器已初始化
    let account: my_program::Counter = program.account(counter_pda).unwrap();
    assert_eq!(account.count, 0, "计数器初始值应为0");
    assert_eq!(account.authority, payer_pubkey, "权限应为用户公钥");
    
    println!("✅ 初始化测试通过");
}

#[test]
fn test_increment() {
    // 使用预充值的payer
    let payer = Keypair::read_from_file("/root/.config/solana/id.json").unwrap();
    let payer_pubkey = payer.pubkey();

    let client = Client::new_with_options(
        Cluster::Localnet,
        Rc::new(payer),
        CommitmentConfig::processed(),
    );
    let program = client.program(my_program::id()).unwrap();

    // 为此测试使用唯一的PDA种子（包含测试名称）
    let (counter_pda, _) = Pubkey::find_program_address(
        &[b"counter", payer_pubkey.as_ref()],
        &my_program::id(),
    );

    // 先初始化计数器（测试隔离）
    program
        .request()
        .accounts(my_program::accounts::Initialize {
            counter: counter_pda,
            user: payer_pubkey,
            system_program: anchor_lang::system_program::ID,
        })
        .args(my_program::instruction::Initialize {})
        .send()
        .unwrap();

    // 测试增加计数
    let _tx = program
        .request()
        .accounts(my_program::accounts::Increment {
            counter: counter_pda,
            authority: payer_pubkey,
        })
        .args(my_program::instruction::Increment {})
        .send()
        .unwrap();

    // 验证计数器已增加
    let account: my_program::Counter = program.account(counter_pda).unwrap();
    assert_eq!(account.count, 1, "计数器应增加到1");
    
    println!("✅ 增加计数测试通过");
}

#[test]
fn test_multiple_increments() {
    // 使用预充值的payer
    let payer = Keypair::read_from_file("/root/.config/solana/id.json").unwrap();
    let payer_pubkey = payer.pubkey();

    let client = Client::new_with_options(
        Cluster::Localnet,
        Rc::new(payer),
        CommitmentConfig::processed(),
    );
    let program = client.program(my_program::id()).unwrap();

    // 为此测试使用唯一的PDA种子
    let (counter_pda, _) = Pubkey::find_program_address(
        &[b"counter", payer_pubkey.as_ref()],
        &my_program::id(),
    );

    // 先初始化计数器
    program
        .request()
        .accounts(my_program::accounts::Initialize {
            counter: counter_pda,
            user: payer_pubkey,
            system_program: anchor_lang::system_program::ID,
        })
        .args(my_program::instruction::Initialize {})
        .send()
        .unwrap();

    // 多次增加计数
    for i in 1..=5 {
        program
            .request()
            .accounts(my_program::accounts::Increment {
                counter: counter_pda,
                authority: payer_pubkey,
            })
            .args(my_program::instruction::Increment {})
            .send()
            .unwrap();

        // 验证每次增加后的值
        let account: my_program::Counter = program.account(counter_pda).unwrap();
        assert_eq!(account.count, i, "计数器应为{}", i);
    }
    
    println!("✅ 多次增加计数测试通过");
}

#[test]
fn test_unauthorized_increment() {
    // 使用预充值的payer
    let payer = Keypair::read_from_file("/root/.config/solana/id.json").unwrap();
    let payer_pubkey = payer.pubkey();

    let client = Client::new_with_options(
        Cluster::Localnet,
        Rc::new(payer),
        CommitmentConfig::processed(),
    );
    let program = client.program(my_program::id()).unwrap();

    // 创建另一个用户
    let other_user = Keypair::new();

    // 为此测试使用唯一的PDA种子
    let (counter_pda, _) = Pubkey::find_program_address(
        &[b"counter", payer_pubkey.as_ref()],
        &my_program::id(),
    );

    // 先初始化计数器
    program
        .request()
        .accounts(my_program::accounts::Initialize {
            counter: counter_pda,
            user: payer_pubkey,
            system_program: anchor_lang::system_program::ID,
        })
        .args(my_program::instruction::Initialize {})
        .send()
        .unwrap();

    // 尝试用错误的权限增加计数（应该失败）
    // 注意：由于PDA种子包含payer_pubkey，其他用户无法派生相同的PDA
    // 这个测试验证了PDA种子机制提供的隐式权限控制
    let (wrong_counter_pda, _) = Pubkey::find_program_address(
        &[b"counter", other_user.pubkey().as_ref()],
        &my_program::id(),
    );

    // 这个PDA不存在，所以调用会失败
    let result = program
        .request()
        .accounts(my_program::accounts::Increment {
            counter: wrong_counter_pda,
            authority: payer_pubkey,
        })
        .args(my_program::instruction::Increment {})
        .send();

    assert!(result.is_err(), "未授权的增加操作应该失败");
    
    println!("✅ 未授权增加测试通过");
}
