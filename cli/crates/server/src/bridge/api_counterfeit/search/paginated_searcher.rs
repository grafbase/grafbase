use std::cmp;
use std::fmt::Debug;

use integer_encoding::{VarIntReader, VarIntWriter};
use tantivy::{self, collector::Count, collector::TopDocs, schema::Field, Document};
use tantivy::{DocAddress, Searcher};

use super::{BadRequestError, Cursor, Hit, Info, PaginatedHits, QueryError, SearchResult};

type DocId = Vec<u8>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchCursor {
    offset: usize,
    doc_id: DocId,
}

#[derive(Debug)]
struct Match {
    offset: usize,
    score: f32,
    doc_id: DocId,
}

impl TryFrom<Cursor> for SearchCursor {
    type Error = QueryError;

    fn try_from(value: Cursor) -> Result<Self, Self::Error> {
        let bytes = value.into_bytes();
        let mut cursor = std::io::Cursor::new(&bytes);
        #[allow(clippy::cast_possible_truncation)]
        Ok(SearchCursor {
            offset: cursor
                .read_varint()
                .map_err(|_| BadRequestError::InvalidCursor(Cursor::from(&bytes[..])))?,
            doc_id: DocId::from(&bytes[(cursor.position() as usize)..]),
        })
    }
}

impl From<SearchCursor> for Cursor {
    fn from(value: SearchCursor) -> Self {
        // We don't need anything more advanced as cursors are short-lived, no need for backwards
        // compatibility here.
        let mut bytes = vec![];
        bytes.write_varint(value.offset).unwrap();
        bytes.extend(value.doc_id);
        Cursor::from(bytes)
    }
}

impl<Id: From<DocId>> From<Match> for Hit<Id> {
    fn from(Match { offset, doc_id, score }: Match) -> Self {
        Hit {
            id: Id::from(doc_id.clone()),
            score,
            cursor: Cursor::from(SearchCursor { offset, doc_id }),
        }
    }
}

pub struct TopDocsPaginatedSearcher {
    pub searcher: Searcher,
    pub query: Box<dyn tantivy::query::Query>,
    pub id_field: Field,
    pub pagination_limit: usize,
}

impl TopDocsPaginatedSearcher {
    pub fn search_forward<Id: From<DocId> + Debug>(&self, first: usize) -> SearchResult<PaginatedHits<Id>> {
        let (total_hits, matches) = self.searcher.search(
            &self.query,
            &(Count, TopDocs::with_limit(cmp::min(self.pagination_limit, first))),
        )?;
        matches
            .into_iter()
            .enumerate()
            .map(|(offset, (score, doc_address))| {
                let doc = self.searcher.doc(doc_address)?;
                get_document_id(&doc, self.id_field).map(|doc_id| Hit::from(Match { offset, score, doc_id }))
            })
            .collect::<SearchResult<Vec<_>>>()
            .map(|hits| {
                let has_next_page = hits.len() < cmp::min(total_hits, self.pagination_limit);
                PaginatedHits {
                    hits,
                    info: Info {
                        has_next_page,
                        has_previous_page: false,
                        total_hits: total_hits as u64,
                    },
                }
            })
    }

    pub fn search_forward_after<Id: From<DocId> + Debug>(
        &self,
        first: usize,
        after: &SearchCursor,
    ) -> SearchResult<PaginatedHits<Id>> {
        let error_margin = cmp::max(1, cmp::min(after.offset >> 4, first >> 3));
        let mut limit = cmp::min(after.offset + first + error_margin, self.pagination_limit);
        loop {
            let (total_hits, cursor_offset, mut hits) = self.load::<Id>(first, after, true, limit)?;
            let pagination_limit = cmp::min(total_hits, self.pagination_limit);

            // Enough hits / no more data
            if hits.len() >= first || limit >= pagination_limit {
                let has_next_page = hits.len() > first || limit < pagination_limit;
                let has_previous_page = cursor_offset.map(|offset| offset > 0).unwrap_or_default();
                hits.truncate(first);
                break Ok(PaginatedHits {
                    hits,
                    info: Info {
                        has_next_page,
                        has_previous_page,
                        total_hits: total_hits as u64,
                    },
                });
            }
            // increase by 50%
            limit = limit + (limit >> 1);
        }
    }

    pub fn search_backward_before<Id: From<DocId> + Debug>(
        &self,
        last: usize,
        before: &SearchCursor,
    ) -> SearchResult<PaginatedHits<Id>> {
        let error_margin = cmp::max(1, cmp::min(before.offset >> 4, last >> 3));
        let mut limit = cmp::min(before.offset + error_margin, self.pagination_limit);
        loop {
            let (total_hits, cursor_offset, mut hits) = self.load::<Id>(last, before, false, limit)?;
            let pagination_limit = cmp::min(total_hits, self.pagination_limit);

            if let Some(cursor_offset) = cursor_offset {
                let has_previous_page = hits.len() > last;
                let has_next_page = cursor_offset < (pagination_limit - 1);
                hits.truncate(last);
                // Reversing to retrieve the original ordering
                hits.reverse();
                break Ok(PaginatedHits {
                    hits,
                    info: Info {
                        has_next_page,
                        has_previous_page,
                        total_hits: total_hits as u64,
                    },
                });
            }
            // Nothing left
            if limit >= pagination_limit {
                break Ok(PaginatedHits {
                    hits: vec![],
                    info: Info {
                        has_previous_page: false,
                        has_next_page: false,
                        total_hits: total_hits as u64,
                    },
                });
            }
            // increase by 50%
            limit = limit + (limit >> 1);
        }
    }

    fn load<Id: From<DocId> + Debug>(
        &self,
        count: usize,
        cursor: &SearchCursor,
        forward: bool,
        limit: usize,
    ) -> SearchResult<(usize, Option<usize>, Vec<Hit<Id>>)> {
        let (total_hits, matches) = self
            .searcher
            .search(&self.query, &(Count, TopDocs::with_limit(limit)))?;
        let matches = matches.into_iter().enumerate();
        let (cursor_offset, hits) = if forward {
            self.extract_hits_after_cursor(count, cursor, matches)?
        } else {
            self.extract_hits_after_cursor(count, cursor, matches.rev())?
        };
        Ok((total_hits, cursor_offset, hits))
    }

    fn extract_hits_after_cursor<Id: From<DocId> + Debug, I: IntoIterator<Item = (usize, (f32, DocAddress))>>(
        &self,
        count: usize,
        cursor: &SearchCursor,
        matches: I,
    ) -> SearchResult<(Option<usize>, Vec<Hit<Id>>)> {
        let mut hits = Vec::new();
        let mut cursor_offset: Option<usize> = None;
        for (offset, (score, doc_address)) in matches {
            let doc = self.searcher.doc(doc_address)?;
            let doc_id = get_document_id(&doc, self.id_field)?;
            if cursor_offset.is_some() {
                hits.push(Hit::from(Match { offset, score, doc_id }));
                // Propagage correctly upstream that we retrieved more than expected.
                // This ensures has_next_page/has_previous_page are properly computed.
                if hits.len() > count {
                    break;
                }
            } else if doc_id == cursor.doc_id {
                cursor_offset = Some(offset);
            }
        }
        Ok((cursor_offset, hits))
    }
}

fn get_document_id(doc: &Document, id_field: Field) -> SearchResult<DocId> {
    let id = doc
        .get_first(id_field)
        .ok_or_else(|| "Document is missing 'id' field".to_string())?;
    match id {
        tantivy::schema::Value::Bytes(bytes) => Ok(bytes.clone()),
        x => Err(format!("Unexpected data for 'id': {x:?}").into()),
    }
}
