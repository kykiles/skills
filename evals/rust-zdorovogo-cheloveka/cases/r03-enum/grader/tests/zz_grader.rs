// Скрытые тесты оценщика r03-enum.
use orders::{can_cancel, can_move, is_open, refund, summary, Order, Status};
use std::io::Write;
use std::process::{Command, Stdio};

const ALL: [Status; 6] = [
    Status::New,
    Status::Paid,
    Status::Shipped,
    Status::Delivered,
    Status::Returned,
    Status::Cancelled,
];

#[test]
fn grader_codes_and_titles() {
    assert_eq!(Status::Returned.code(), "returned");
    assert_eq!(Status::from_code("returned"), Some(Status::Returned));
    assert_eq!(Status::Returned.title(), "возвращён");
    for status in ALL {
        assert_eq!(Status::from_code(status.code()), Some(status));
    }
}

#[test]
fn grader_policy() {
    let open = [true, true, true, false, false, false];
    let cancel = [true, true, false, false, false, false];
    for (i, status) in ALL.into_iter().enumerate() {
        assert_eq!(is_open(status), open[i], "is_open({status:?})");
        assert_eq!(can_cancel(status), cancel[i], "can_cancel({status:?})");
    }
}

#[test]
fn grader_transitions() {
    use Status::*;
    let allowed = [
        (New, Paid),
        (Paid, Shipped),
        (Shipped, Delivered),
        (New, Cancelled),
        (Paid, Cancelled),
        (Delivered, Returned),
    ];
    for from in ALL {
        for to in ALL {
            assert_eq!(
                can_move(from, to),
                allowed.contains(&(from, to)),
                "can_move({from:?}, {to:?})"
            );
        }
    }
}

#[test]
fn grader_refund() {
    let order = |status, total| Order {
        id: 1,
        status,
        total,
    };
    assert_eq!(refund(&order(Status::Returned, 12_345)), 12_345);
    assert_eq!(refund(&order(Status::Returned, 0)), 0);
    assert_eq!(refund(&order(Status::Cancelled, 700)), 700);
    assert_eq!(refund(&order(Status::Delivered, 700)), 0);
}

// Сводка без строки «возвращён: N»; строка должна стоять среди строк статусов.
fn split_summary(text: &str, returned: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let wanted = format!("возвращён: {returned}");
    let positions: Vec<usize> = (0..lines.len()).filter(|&i| lines[i] == wanted).collect();
    assert_eq!(
        positions.len(),
        1,
        "нужна ровно одна строка «{wanted}»:\n{text}"
    );
    assert!(
        positions[0] < 6,
        "строка «{wanted}» не среди строк статусов:\n{text}"
    );
    let mut rest: Vec<&str> = lines.clone();
    rest.remove(positions[0]);
    rest.join("\n") + "\n"
}

#[test]
fn grader_summary() {
    let orders = [
        Order {
            id: 1,
            status: Status::Paid,
            total: 15000,
        },
        Order {
            id: 2,
            status: Status::Returned,
            total: 2000,
        },
        Order {
            id: 3,
            status: Status::Delivered,
            total: 100000,
        },
        Order {
            id: 4,
            status: Status::Returned,
            total: 50,
        },
        Order {
            id: 5,
            status: Status::Cancelled,
            total: 9950,
        },
    ];
    let got = split_summary(&summary(&orders), 2);
    assert_eq!(
        got,
        "новый: 0\nоплачен: 1\nв доставке: 0\nдоставлен: 1\nотменён: 1\n\
         открытых: 1, закрытых: 4\nвыручка: 1150.00 ₽\nвозвраты: 120.00 ₽\n"
    );
    let empty = split_summary(&summary(&[]), 0);
    assert_eq!(
        empty,
        "новый: 0\nоплачен: 0\nв доставке: 0\nдоставлен: 0\nотменён: 0\n\
         открытых: 0, закрытых: 0\nвыручка: 0.00 ₽\nвозвраты: 0.00 ₽\n"
    );
}

#[test]
fn grader_cli() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_orders"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"1 delivered 500\n2 returned 300\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{}\n{}",
        stdout,
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        split_summary(&stdout, 1),
        "новый: 0\nоплачен: 0\nв доставке: 0\nдоставлен: 1\nотменён: 0\n\
         открытых: 0, закрытых: 2\nвыручка: 5.00 ₽\nвозвраты: 3.00 ₽\n"
    );
}
