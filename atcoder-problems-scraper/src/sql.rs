pub mod query;

use crate::{Contest, Problem, Submission};
use postgres::{Connection, TlsMode};

pub struct SqlClient {
    user: String,
    pass: String,
    host: String,
    db: String,
}

impl SqlClient {
    pub fn new(user: &str, pass: &str, host: &str, db: &str) -> Self {
        Self {
            user: user.to_owned(),
            pass: pass.to_owned(),
            host: host.to_owned(),
            db: db.to_owned(),
        }
    }

    fn connect(&self) -> Result<Connection, String> {
        Connection::connect(
            format!(
                "postgresql://{}:{}@{}/{}",
                self.user, self.pass, self.host, self.db
            ),
            TlsMode::None,
        )
        .map_err(|e| format!("{:?}", e))
    }

    pub fn insert_submissions(&self, submissions: &[Submission]) -> Result<Vec<u64>, String> {
        let conn = self.connect()?;
        let query = r"
        INSERT INTO submissions (
            id,
            epoch_second,
            problem_id,
            contest_id,
            user_id,
            language,
            point,
            length,
            result,
            execution_time
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (id) DO UPDATE SET user_id = $5
        ";
        let statement = conn.prepare(query).map_err(|e| format!("{:?}", e))?;
        submissions
            .iter()
            .map(|submission| {
                statement
                    .execute(&[
                        &submission.id,
                        &submission.epoch_second,
                        &submission.problem_id,
                        &submission.contest_id,
                        &submission.user_id,
                        &submission.language,
                        &submission.point,
                        &submission.length,
                        &&submission.result,
                        &submission.execution_time,
                    ])
                    .map_err(|e| format!("{:?}", e))
            })
            .collect()
    }

    pub fn insert_contests(&self, contests: &[Contest]) -> Result<Vec<u64>, String> {
        let conn = self.connect()?;
        let statement = conn
            .prepare(
                r"
            INSERT INTO contests (id, start_epoch_second, duration_second, title, rate_change)
            VALUES ($1, $2, $3, $4, $5) ON CONFLICT (id) DO NOTHING
        ",
            )
            .map_err(|e| format!("{:?}", e))?;
        contests
            .iter()
            .map(|contest| {
                statement
                    .execute(&[
                        &contest.id,
                        &contest.start_epoch_second,
                        &contest.duration_second,
                        &contest.title,
                        &contest.rate_change,
                    ])
                    .map_err(|e| format!("{:?}", e))
            })
            .collect()
    }

    pub fn insert_problems(&self, problems: &[Problem]) -> Result<Vec<u64>, String> {
        let conn = self.connect()?;
        let statement = conn
            .prepare(
                r"
            INSERT INTO problems (id, contest_id, title)
            VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING
        ",
            )
            .map_err(|e| format!("{:?}", e))?;
        problems
            .iter()
            .map(|problem| {
                statement
                    .execute(&[&problem.id, &problem.contest_id, &problem.title])
                    .map_err(|e| format!("{:?}", e))
            })
            .collect()
    }

    pub fn get_problems(&self) -> Result<Vec<Problem>, String> {
        let conn = self.connect()?;
        conn.query("SELECT id, contest_id, title FROM problems", &[])
            .map_err(|e| format!("{:?}", e))?
            .into_iter()
            .map(|row| {
                Ok(Problem {
                    id: row.get("id"),
                    contest_id: row.get("contest_id"),
                    title: row.get("title"),
                })
            })
            .collect()
    }

    pub fn get_contests(&self) -> Result<Vec<Contest>, String> {
        let conn = self.connect()?;
        conn.query(
            "SELECT id, start_epoch_second, duration_second, title, rate_change FROM contests",
            &[],
        )
        .map_err(|e| format!("{:?}", e))?
        .into_iter()
        .map(|row| {
            Ok(Contest {
                id: row.get("id"),
                start_epoch_second: row.get("start_epoch_second"),
                duration_second: row.get("duration_second"),
                title: row.get("title"),
                rate_change: row.get("rate_change"),
            })
        })
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;

    const URL: &str = "postgresql://kenkoooo:pass@localhost/test";

    fn read_file(path: &str) -> String {
        let mut file = File::open(path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        contents
    }

    fn setup_test_db() {
        let conn = Connection::connect(URL, TlsMode::None).unwrap();
        let sql = read_file("../config/database-definition.sql");
        conn.batch_execute(&sql).unwrap();
    }

    fn connect_to_test() -> SqlClient {
        SqlClient {
            user: "kenkoooo".to_owned(),
            pass: "pass".to_owned(),
            host: "localhost".to_owned(),
            db: "test".to_owned(),
        }
    }

    #[test]
    fn test_insert_submission() {
        setup_test_db();

        let mut v = vec![Submission {
            id: 0,
            epoch_second: 0,
            problem_id: "".to_owned(),
            contest_id: "".to_owned(),
            user_id: "".to_owned(),
            language: "".to_owned(),
            point: 0.0,
            length: 0,
            result: "".to_owned(),
            execution_time: None,
        }];

        let conn = connect_to_test();
        v[0].id = 1;
        conn.insert_submissions(&v).unwrap();

        let count = Connection::connect(URL, TlsMode::None)
            .unwrap()
            .query("SELECT id FROM submissions", &[])
            .unwrap()
            .into_iter()
            .count();
        assert_eq!(count, 1);

        v[0].id = 2;
        conn.insert_submissions(&v).unwrap();
        let count = Connection::connect(URL, TlsMode::None)
            .unwrap()
            .query("SELECT id FROM submissions", &[])
            .unwrap()
            .into_iter()
            .count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_update_submission() {
        setup_test_db();

        let mut v = vec![Submission {
            id: 0,
            epoch_second: 0,
            problem_id: "".to_owned(),
            contest_id: "".to_owned(),
            user_id: "".to_owned(),
            language: "".to_owned(),
            point: 0.0,
            length: 0,
            result: "".to_owned(),
            execution_time: None,
        }];

        let conn = connect_to_test();

        v[0].user_id = "kenkoooo".to_owned();
        conn.insert_submissions(&v).unwrap();
        let user_id: String = Connection::connect(URL, TlsMode::None)
            .unwrap()
            .query("SELECT user_id FROM submissions", &[])
            .unwrap()
            .into_iter()
            .next()
            .unwrap()
            .get(0);
        assert_eq!(user_id, "kenkoooo".to_owned());

        v[0].user_id = "ooooknek".to_owned();
        conn.insert_submissions(&v).unwrap();
        let user_id: String = Connection::connect(URL, TlsMode::None)
            .unwrap()
            .query("SELECT user_id FROM submissions", &[])
            .unwrap()
            .into_iter()
            .next()
            .unwrap()
            .get(0);
        assert_eq!(user_id, "ooooknek".to_owned());
    }

    #[test]
    fn test_insert_problems() {
        setup_test_db();
        let conn = connect_to_test();

        let count = Connection::connect(URL, TlsMode::None)
            .unwrap()
            .query("SELECT id FROM problems", &[])
            .unwrap()
            .into_iter()
            .count();
        assert_eq!(count, 0);

        let problems = vec![
            Problem {
                id: "arc001_a".to_owned(),
                contest_id: "arc001".to_owned(),
                title: "Problem 1".to_owned(),
            },
            Problem {
                id: "arc001_b".to_owned(),
                contest_id: "arc001".to_owned(),
                title: "Problem 2".to_owned(),
            },
        ];
        conn.insert_problems(&problems).unwrap();

        let count = Connection::connect(URL, TlsMode::None)
            .unwrap()
            .query("SELECT id FROM problems", &[])
            .unwrap()
            .into_iter()
            .count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_insert_contests() {
        setup_test_db();
        let conn = connect_to_test();

        let count = Connection::connect(URL, TlsMode::None)
            .unwrap()
            .query("SELECT id FROM contests", &[])
            .unwrap()
            .into_iter()
            .count();
        assert_eq!(count, 0);

        let contests = vec![
            Contest {
                id: "arc001".to_owned(),
                start_epoch_second: 0,
                duration_second: 0,
                title: "Contest 1".to_owned(),
                rate_change: "-".to_owned(),
            },
            Contest {
                id: "arc002".to_owned(),
                start_epoch_second: 0,
                duration_second: 0,
                title: "Contest 2".to_owned(),
                rate_change: "-".to_owned(),
            },
        ];
        conn.insert_contests(&contests).unwrap();

        let count = Connection::connect(URL, TlsMode::None)
            .unwrap()
            .query("SELECT id FROM contests", &[])
            .unwrap()
            .into_iter()
            .count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_get_contests_problems() {
        setup_test_db();
        Connection::connect(URL, TlsMode::None)
            .unwrap()
            .batch_execute(
                r"
            INSERT INTO contests (id, start_epoch_second, duration_second, title, rate_change)
            VALUES (1, 0, 0, 'Contest 1', '-'), (2, 0, 0, 'Contest 2', '-'), (3, 0, 0, 'Contest 3', '-');",
            )
            .unwrap();

        let conn = connect_to_test();
        let contests = conn.get_contests().unwrap();
        assert_eq!(
            vec![
                Contest {
                    id: "1".to_owned(),
                    start_epoch_second: 0,
                    duration_second: 0,
                    title: "Contest 1".to_owned(),
                    rate_change: "-".to_owned()
                },
                Contest {
                    id: "2".to_owned(),
                    start_epoch_second: 0,
                    duration_second: 0,
                    title: "Contest 2".to_owned(),
                    rate_change: "-".to_owned()
                },
                Contest {
                    id: "3".to_owned(),
                    start_epoch_second: 0,
                    duration_second: 0,
                    title: "Contest 3".to_owned(),
                    rate_change: "-".to_owned()
                }
            ],
            contests
        );

        Connection::connect(URL, TlsMode::None)
            .unwrap()
            .batch_execute(
                r"
            INSERT INTO problems (id, contest_id, title)
            VALUES ('problem_a', 'contest_a', 'Problem A'), ('problem_b', 'contest_a', 'Problem B'), ('problem_z', 'contest_b', 'Problem Z');",
            )
            .unwrap();

        let conn = connect_to_test();
        let problems = conn.get_problems().unwrap();
        assert_eq!(
            vec![
                Problem {
                    id: "problem_a".to_owned(),
                    contest_id: "contest_a".to_owned(),
                    title: "Problem A".to_owned()
                },
                Problem {
                    id: "problem_b".to_owned(),
                    contest_id: "contest_a".to_owned(),
                    title: "Problem B".to_owned()
                },
                Problem {
                    id: "problem_z".to_owned(),
                    contest_id: "contest_b".to_owned(),
                    title: "Problem Z".to_owned()
                }
            ],
            problems
        );
    }
}
