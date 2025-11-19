use std::sync::Arc;

use tokio::sync::{Mutex, OnceCell};

use fern_sdk::bindings::{
    self,
    exports::fern::base::guest::{Guest, TickError},
    fern::base::sqlite::{Database, SqliteValue},
    wasi::{self, clocks::wall_clock::Datetime},
};

static DB: tokio::sync::OnceCell<Database> = OnceCell::const_new();

async fn db<'a>() -> &'a Database {
    DB.get_or_init(|| async {
        fern_sdk::bindings::fern::base::sqlite::open_db("heck".to_string())
            .await
            .expect("failed to open db")
    })
    .await
}

static GUEST: tokio::sync::OnceCell<Arc<Mutex<SampleGuest>>> = OnceCell::const_new();

async fn guest<'a>() -> Arc<Mutex<SampleGuest>> {
    GUEST
        .get_or_init(|| async { Arc::new(Mutex::new(SampleGuest { date_time: None })) })
        .await
        .clone()
}

struct SampleGuest {
    date_time: Option<Datetime>,
}

impl Guest for SampleGuest {
    #[allow(async_fn_in_trait)]
    async fn init() -> bool {
        let db = db().await;

        let r = db
            .execute(
                "CREATE TABLE users (
      id INTEGER PRIMARY KEY,
      name TEXT NOT NULL,
      email TEXT NOT NULL,
      created_at INTEGER NOT NULL
    )"
                .into(),
                vec![],
            )
            .await
            .unwrap();
        println!("{r:#?}");

        db.execute(
            "INSERT INTO users (name, email, created_at) VALUES (?, ?, ?)".into(),
            vec![
                "Alice".into(),
                "alice@example.com".into(),
                1700000000.into(),
            ],
        )
        .await
        .unwrap();

        db.execute(
            "INSERT INTO users (name, email, created_at) VALUES (?, ?, ?)".into(),
            vec!["Bob".into(), "bob@example.com".into(), 12345.into()],
        )
        .await
        .unwrap();

        true
    }

    #[allow(async_fn_in_trait)]
    async fn post_init() -> bool {
        let db = db().await;
        let rows = db
            .query(
                "SELECT id, name, email FROM users ORDER BY id".to_string(),
                vec![],
            )
            .await
            .unwrap();
        let mut data: Vec<_> = vec![];
        while let Ok(Some(row)) = rows.next().await {
            data.push(row);
        }

        assert_eq!(data.len(), 2);
        assert_eq!(
            &SqliteValue::Text("Alice".to_string()),
            data.get(0).unwrap().values.get(1).unwrap()
        );
        assert_eq!(
            &SqliteValue::Text("Bob".to_string()),
            data.get(1).unwrap().values.get(1).unwrap()
        );

        true
    }

    #[allow(async_fn_in_trait)]
    async fn shutdown() -> bool {
        true
    }

    #[allow(async_fn_in_trait)]
    async fn tick() -> Result<(), TickError> {
        let now = wasi::clocks::wall_clock::now().await;

        let guest = guest().await;
        let mut guest = guest.lock().await;

        if guest.date_time.is_none() {
            guest.date_time.replace(now);
        } else {
            let old = guest.date_time.take().unwrap();
            println!("Time between calls {}s+{}ns", now.seconds - old.seconds, now.nanoseconds - old.nanoseconds);
            guest.date_time.replace(now);
        }
        Ok(())
    }
}

bindings::export!(SampleGuest with_types_in bindings);
