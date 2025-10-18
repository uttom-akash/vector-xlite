use regex::Regex;
use rusqlite::{Connection, Result};
use rusqlite::{LoadExtensionGuard, Rows};

pub struct VectorXSqlite {
    pub conn: Connection,
}

impl VectorXSqlite {
    fn load_my_extension(conn: &Connection) -> Result<()> {
        unsafe {
            let _guard = LoadExtensionGuard::new(conn)?;
            conn.load_extension("/home/akash/Dev/artifacts/vectorlite_py-0.2.0-py3-none-manylinux_2_17_x86_64.manylinux2014_x86_64/vectorlite_py/vectorlite.so",
         None::<&str>)
        }
    }

    fn get_virtual_table_name(table_name: &str) -> String {
        format!("vt_{}", table_name)
    }

    fn replace_select_with_count(query: &str) -> String {
        // Regex: match SELECT ... FROM (non-greedy)
        let re = Regex::new(r"(?i)^SELECT\s+.*?\s+FROM").unwrap();
        // Replace with SELECT count(*) FROM
        re.replace(query, "SELECT count(*) FROM").to_string()
    }

    fn replace_select_with_row_ids(query: &str) -> String {
        // Regex: match SELECT ... FROM (non-greedy)
        let re = Regex::new(r"(?i)^SELECT\s+.*?\s+FROM").unwrap();
        // Replace with SELECT count(*) FROM
        re.replace(query, "SELECT rowid FROM").to_string()
    }

    pub fn new() -> Result<Self> {
        let conn = Connection::open_in_memory()?;

        Self::load_my_extension(&conn)?;

        Ok(VectorXSqlite { conn })
    }

    pub fn create_collection(&self, collection_name: &str, payload_table: &str) -> Result<()> {
        let virtual_table_query = format!(
            "create virtual table {} using vectorlite(my_embedding float32[3], hnsw(max_elements=100))",
            Self::get_virtual_table_name(collection_name)
        );

        self.conn.execute(
            virtual_table_query.as_str(),
            (), // empty list of parameters.
        )?;

        self.conn.execute(payload_table, ())?;

        Ok(())
    }

    pub fn insert_vector(
        &self,
        collection_name: &str,
        rowid: i64,
        vector: &Vec<f32>,
        payload: &str,
    ) -> Result<()> {
        let insert_query = format!(
            "insert into {}(rowid, my_embedding) values (?, vector_from_json(?))",
            Self::get_virtual_table_name(collection_name)
        );

        let vector_json = format!("{:?}", vector);

        self.conn
            .execute(&insert_query, (rowid, vector_json.as_str()))?;

        self.conn.execute(payload, (rowid,))?;

        Ok(())
    }

    pub fn search_vectors(
        &self,
        collection_name: &str,
        query_vector: &Vec<f32>,
        top_k: i64,
        payload: &str,
    ) -> Result<Vec<(i64, f32)>> {
        let vector_json = format!("{:?}", query_vector);

        let payload_selection_count = self
            .conn
            .query_one(&Self::replace_select_with_count(payload), (), |row| {
                let count: i64 = row.get(0)?;
                Ok(count)
            })
            .unwrap_or(0);

        let mut results = Vec::new();
        // 1. ids : attribute query in full database [when more selective]
        // 2. distance, ids : vector search in subset ids from 1
        if payload_selection_count < 10000 {
            let search_query = format!(
                "SELECT vt.rowid, vt.distance, pt.*
                    FROM (
                        SELECT vt_inner.rowid, vt_inner.distance
                        FROM {vt_table_name} as vt_inner
                        WHERE knn_search(vt_inner.my_embedding, knn_param(vector_from_json(?1), ?2)) 
                        AND vt_inner.rowid in ({payload_query_ids})
                    ) AS vt
                INNER JOIN ({payload_query}) AS pt
                    ON vt.rowid = pt.rowid 
                    ORDER BY vt.distance LIMIT ?2",
                payload_query_ids = Self::replace_select_with_row_ids(payload),
                vt_table_name = Self::get_virtual_table_name(collection_name),
                payload_query = payload,
            );

            let mut stmt = self.conn.prepare(&search_query)?;
            let mut rows = stmt.query((vector_json.as_str(), top_k))?;
            while let Some(row) = rows.next()? {
                let rowid: i64 = row.get(0)?;
                let distance: f32 = row.get(1)?;
                results.push((rowid, distance));
            }
        } else {
            // 1. distance, ids : vector search in full index with over sampling
            // 2. ids : attribute query in subset ids from 1 [when less selective]
            let search_query = format!(
                "SELECT vt.rowid, vt.distance, pt.*
                FROM (
                    SELECT vt_inner.rowid, vt_inner.distance
                    FROM {collection} as vt_inner
                    WHERE knn_search(vt_inner.my_embedding, knn_param(vector_from_json(?1), ?2))
                ) AS vt
                INNER JOIN ({payload_query}) AS pt
                    ON vt.rowid = pt.rowid 
                    ORDER BY vt.distance LIMIT ?3 ",
                collection = Self::get_virtual_table_name(collection_name),
                payload_query = payload,
            );

            let mut stmt = self.conn.prepare(&search_query)?;
            let mut rows = stmt.query((vector_json.as_str(), 10 * top_k, top_k))?;
            while let Some(row) = rows.next()? {
                let rowid: i64 = row.get(0)?;
                let distance: f32 = row.get(1)?;
                results.push((rowid, distance));
            }
        }

        Ok(results)
    }

    pub fn hihello(&self) -> Result<()> {
        self.conn.execute(
            "create virtual table my_table using vectorlite(my_embedding float32[3], hnsw(max_elements=100))",
            (), // empty list of parameters.
        )?;

        self.conn.execute(
            "insert into my_table(rowid, my_embedding) values (0, vector_from_json('[1,2,3]'))",
            (),
        )?;

        self.conn.execute(
            "insert into my_table(rowid, my_embedding) values (2, vector_from_json('[7,7,7]'))",
            (),
        )?;

        self.conn.execute(
            "insert into my_table(rowid, my_embedding) values (1, vector_from_json('[2,3,4]'))",
            (),
        )?;

        let mut stmt = self
            .conn
            .prepare("SELECT sql FROM sqlite_master WHERE type='table' AND name=?1;")?;
        let mut rows = stmt.query(["my_table"])?;

        if let Some(row) = rows.next()? {
            let schema: String = row.get(0)?;
            println!("Schema:\n{}", schema);
        }

        let mut stmt = self.conn.prepare("select rowid, distance, my_embedding from my_table where knn_search(my_embedding, knn_param(vector_from_json('[3,4,5]'), 3))")?;
        let mut result = stmt.query([]).unwrap();

        loop {
            let row = result.next().unwrap();

            match row {
                Some(r) => {
                    // let info: String = r.unwrap();
                    println!("Vectorlite info: {:?}", r);
                }
                None => {
                    break;
                }
            }
        }

        Ok(())
        // let person_iter = stmt.query_map([], |row| {

        //     -- Load vectorlite
        // .load path/to/vectorlite.[so|dll|dylib]
        // -- shows vectorlite version and build info.
        // select vectorlite_info();
        // -- Calculate vector l2(squared) distance
        // select vector_distance(vector_from_json('[1,2,3]'), vector_from_json('[3,4,5]'), 'l2');
        // -- Create a virtual table named my_table with one vector column my_embedding with dimention of 3
        // create virtual table my_table using vectorlite(my_embedding float32[3], hnsw(max_elements=100));
        // -- Insert vectors into my_table. rowid can be used to relate to a vector's metadata stored elsewhere, e.g. another table.
        // insert into my_table(rowid, my_embedding) values (0, vector_from_json('[1,2,3]'));
        // insert into my_table(rowid, my_embedding) values (1, vector_from_json('[2,3,4]'));
        // insert into my_table(rowid, my_embedding) values (2, vector_from_json('[7,7,7]'));
        // -- Find 2 approximate nearest neighbors of vector [3,4,5] with distances
        // select rowid, distance from my_table where knn_search(my_embedding, knn_param(vector_from_json('[3,4,5]'), 2));
        // -- Find the nearest neighbor of vector [3,4,5] among vectors with rowid 0 and 1. (requires sqlite_version>=3.38)
        // -- It is called metadata filter in vectorlite, because you could get rowid set beforehand based on vectors' metadata and then perform vector search.
        // -- Metadata filter is pushed down to the underlying index when traversing the HNSW graph.
        // select rowid, distance from my_table where knn_search(my_embedding, knn_param(vector_from_json('[3,4,5]'), 1)) and rowid in (0, 1) ;
    }
}
