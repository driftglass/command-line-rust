use assert_cmd::Command;

#[test]
fn works() {
    assert!(true);
}

#[test]
fn runs() {
    let res = Command::cargo_bin("hello");
    if let Ok(mut cmd) = res {
        cmd.assert().success().stdout("Hello, world!\n");
    }
    // エラー処理は省略
}

#[test]
fn true_ok() {
    let res = Command::cargo_bin("true");
    if let Ok(mut cmd) = res {
        cmd.assert().success();
    }
    // エラー処理は省略
}

#[test]
fn false_not_ok() {
    let res = Command::cargo_bin("false");
    if let Ok(mut cmd) = res {
        cmd.assert().failure(); // コマンドが失敗(1を返す)したことを確認
    }
    // エラー処理は省略
}
