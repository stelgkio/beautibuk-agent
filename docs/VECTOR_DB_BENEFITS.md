# Vector Database Benefits & Business Data Storage

## Benefits of Using Vector Database (PostgreSQL + pgvector)

### 1. **Semantic Search & Understanding**
- **Keyword vs Semantic**: Traditional search matches keywords, vector search understands meaning
  - Example: User says "I need my hair done" → Vector DB finds businesses with "haircut", "styling", "hair salon" even if they don't contain "hair done"
  
- **Multilingual Support**: Embeddings capture meaning across languages
  - "Κόψιμο μαλλιών" (Greek) and "haircut" (English) have similar embeddings
  - No need for manual translation mappings

### 2. **Improved Business Matching**
- **Fuzzy Matching**: Find businesses even with typos or variations
  - "beauty salon" matches "beauty parlor", "salon de beauté", "beauty center"
  
- **Intent-Based Search**: Understand user intent, not just words
  - "I want to relax" → Finds spas, massage parlors, wellness centers
  - "I need a trim" → Finds hair salons, barbershops

### 3. **RAG (Retrieval Augmented Generation) for Businesses**
- **Context-Aware Recommendations**: Agent can retrieve similar businesses from past conversations
- **Better Suggestions**: "Based on your previous searches for high-end salons, here are similar options..."
- **Learning from Patterns**: Understand what types of businesses users prefer

### 4. **Conversation Context Enhancement**
- **Similar Conversation Retrieval**: Find past conversations with similar intents
  - When user says "I need a haircut", retrieve how previous users successfully booked haircuts
  - Learn from successful booking patterns
  
- **Better Response Generation**: LLM has examples of similar interactions
- **Reduced Hallucination**: Ground responses in actual past conversations

### 5. **Performance & Scalability**
- **Fast Semantic Search**: Vector indexes (IVFFlat/HNSW) enable fast similarity search
- **Handles Large Datasets**: Efficient even with millions of businesses
- **Single Database**: PostgreSQL handles both relational and vector data

## Storing Business Data in Vector DB: Options & Trade-offs

### Option 1: **Hybrid Approach (Recommended)**

Keep business data in MongoDB (Python backend), but create embeddings in PostgreSQL.

**Architecture**:
```
MongoDB (Python Backend)          PostgreSQL (Rust Agent)
├── Business documents            ├── Business embeddings
├── Full business data           ├── business_id (reference)
├── Services, bookings, etc.     ├── Name, description embeddings
└── Source of truth              └── For semantic search only
```

**Benefits**:
- ✅ Single source of truth (MongoDB)
- ✅ No data duplication issues
- ✅ Business data stays in Python backend
- ✅ Embeddings for semantic search only
- ✅ Easy to update embeddings when business data changes

**Implementation**:
```rust
// When business data is fetched from Python backend
// Store embedding in PostgreSQL for semantic search

struct BusinessEmbedding {
    business_id: String,        // Reference to MongoDB
    name_embedding: Vec<f32>,   // Embedding of business name
    description_embedding: Vec<f32>, // Embedding of description
    full_text_embedding: Vec<f32>,   // Combined embedding
}
```

**Changes Required**:
1. Create `business_embeddings` table (already in schema)
2. Generate embeddings when businesses are searched/fetched
3. Store embeddings in PostgreSQL
4. Use embeddings for semantic search, then fetch full data from MongoDB

### Option 2: **Full Business Data in Vector DB**

Store complete business data in PostgreSQL alongside embeddings.

**Architecture**:
```
MongoDB (Python Backend)          PostgreSQL (Rust Agent)
├── Business documents            ├── Business full data
├── Services, bookings, etc.     ├── Business embeddings
└── Source of truth              ├── Mirrored business data
                                  └── For agent's use
```

**Benefits**:
- ✅ Faster access for agent (no API call needed)
- ✅ Can do complex joins with conversations
- ✅ Single database for agent operations

**Drawbacks**:
- ❌ Data duplication (two sources of truth)
- ❌ Sync complexity (need to keep MongoDB and PostgreSQL in sync)
- ❌ Storage overhead (duplicate data)
- ❌ Sync failures could cause inconsistencies

**Changes Required**:
1. Create full business schema in PostgreSQL
2. Implement sync mechanism from MongoDB to PostgreSQL
3. Handle updates, deletes, creates
4. Deal with sync failures and retries
5. Maintain data consistency

### Option 3: **Embeddings Only (Recommended for MVP)**

Store only embeddings, fetch full data from Python backend when needed.

**Architecture**:
```
MongoDB (Python Backend)          PostgreSQL (Rust Agent)
├── Business documents            ├── Business embeddings only
├── Full business data           ├── business_id reference
└── Source of truth              └── Minimal storage
```

**Benefits**:
- ✅ Minimal storage
- ✅ No sync complexity
- ✅ Fast semantic search
- ✅ Full data always fresh from MongoDB

**Implementation Flow**:
1. User query → Generate embedding
2. Semantic search in PostgreSQL → Find similar business_ids
3. Fetch full business data from Python backend API
4. Return combined results

## Recommended Implementation: Hybrid Approach

### Database Schema

```sql
-- Business embeddings (minimal data + embeddings)
CREATE TABLE business_embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    business_id TEXT NOT NULL UNIQUE,  -- Reference to MongoDB
    business_name TEXT NOT NULL,
    business_type TEXT,
    city TEXT,
    state TEXT,
    -- Embeddings
    name_embedding vector(1536),
    description_embedding vector(1536),
    full_text_embedding vector(1536),  -- Combined: name + description + type
    -- Metadata
    last_synced_at TIMESTAMP DEFAULT NOW(),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Index for semantic search
CREATE INDEX idx_business_embeddings_full_text 
    ON business_embeddings 
    USING ivfflat (full_text_embedding vector_cosine_ops);

-- Service embeddings (optional)
CREATE TABLE service_embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_id TEXT NOT NULL UNIQUE,
    business_id TEXT NOT NULL,
    service_name TEXT NOT NULL,
    service_description TEXT,
    category TEXT,
    embedding vector(1536),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX idx_service_embeddings_vector 
    ON service_embeddings 
    USING ivfflat (embedding vector_cosine_ops);
```

### Implementation Changes

#### 1. **MCP Client Integration** (Replaces Backend API Client)

With MCP, the agent uses MCP client instead of direct HTTP API calls. The LLM autonomously calls MCP tools.

```rust
impl McpClient {
    // When fetching businesses, also store embeddings
    pub async fn search_businesses_with_embeddings(
        &self,
        query: Option<&str>,
        // ... other params
    ) -> Result<BusinessSearchResponse, ApiError> {
        // 1. Fetch businesses from Python backend
        let businesses = self.search_businesses(query, ...).await?;
        
        // 2. Generate and store embeddings
        for business in &businesses.businesses {
            self.store_business_embedding(business).await?;
        }
        
        Ok(businesses)
    }
    
    async fn store_business_embedding(
        &self,
        business: &Business,
    ) -> Result<(), ApiError> {
        // Generate embeddings
        let name_emb = self.embedding_service
            .generate_embedding(&business.name).await?;
        let desc_emb = self.embedding_service
            .generate_embedding(&business.description.unwrap_or_default()).await?;
        
        // Combine for full-text search
        let full_text = format!("{} {} {}", 
            business.name,
            business.description.as_deref().unwrap_or(""),
            business.business_type
        );
        let full_text_emb = self.embedding_service
            .generate_embedding(&full_text).await?;
        
        // Store in PostgreSQL
        self.vector_service.store_business_embedding(
            business.id.clone(),
            business.name.clone(),
            business.business_type.clone(),
            business.city.clone(),
            business.state.clone(),
            name_emb,
            desc_emb,
            full_text_emb,
        ).await?;
        
        Ok(())
    }
}
```

#### 2. **Semantic Business Search**

```rust
impl VectorService {
    pub async fn search_businesses_semantic(
        &self,
        query_embedding: Vec<f32>,
        city: Option<&str>,
        business_type: Option<&str>,
        limit: i64,
    ) -> Result<Vec<BusinessMatch>, sqlx::Error> {
        let query_vector = Vector::from(query_embedding);
        
        let mut query = sqlx::query_as!(
            BusinessMatch,
            r#"
            SELECT 
                business_id,
                business_name,
                business_type,
                city,
                1 - (full_text_embedding <=> $1) as similarity
            FROM business_embeddings
            WHERE 1 - (full_text_embedding <=> $1) > 0.7
            "#,
            query_vector
        );
        
        // Add filters
        if let Some(c) = city {
            query = sqlx::query_as!(
                BusinessMatch,
                r#"
                SELECT 
                    business_id,
                    business_name,
                    business_type,
                    city,
                    1 - (full_text_embedding <=> $1) as similarity
                FROM business_embeddings
                WHERE 1 - (full_text_embedding <=> $1) > 0.7
                  AND city = $2
                ORDER BY full_text_embedding <=> $1
                LIMIT $3
                "#,
                query_vector,
                c,
                limit
            );
        }
        
        let results = query.fetch_all(&self.pool).await?;
        Ok(results)
    }
}
```

#### 3. **Enhanced Agent Flow**

```rust
impl AgentOrchestrator {
    pub async fn process_search_with_semantic(
        &self,
        user_query: &str,
        location: Option<Location>,
    ) -> Result<Vec<Business>, AgentError> {
        // 1. Generate embedding for user query
        let query_embedding = self.embedding_service
            .generate_embedding(user_query).await?;
        
        // 2. Semantic search in PostgreSQL
        let business_matches = self.vector_service
            .search_businesses_semantic(
                query_embedding,
                location.as_ref().map(|l| l.city.as_str()),
                None, // business_type extracted from query
                10,   // limit
            ).await?;
        
        // 3. With MCP, LLM autonomously calls get_business_details for each business_id
        // Or fetch full business data from Python backend via MCP
        let business_ids: Vec<String> = business_matches
            .iter()
            .map(|m| m.business_id.clone())
            .collect();
        
        // Note: With MCP, the LLM would call get_business_details tool for each business_id
        // This is just an example showing the flow
        let businesses = vec![]; // Would be populated via MCP tool calls
        
        // 4. Sort by similarity score
        let mut businesses_with_scores: Vec<_> = businesses
            .into_iter()
            .zip(business_matches.iter())
            .map(|(b, m)| (b, m.similarity))
            .collect();
        
        businesses_with_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        Ok(businesses_with_scores.into_iter().map(|(b, _)| b).collect())
    }
}
```

### Sync Strategy

#### Option A: On-Demand (Lazy Loading)
- Generate embeddings when businesses are first searched
- Cache embeddings in PostgreSQL
- Update embeddings periodically or on-demand

#### Option B: Background Sync
- Periodic job that syncs business data from MongoDB
- Generates embeddings for all businesses
- Keeps PostgreSQL up-to-date

#### Option C: Event-Driven
- Webhook from Python backend when business data changes
- Update embeddings immediately

**Recommended for MVP**: Option A (On-Demand)

## Benefits Summary

### With Business Embeddings:

1. **Better Search Results**
   - User: "I need a place to get my nails done"
   - Finds: "Nail Salon", "Manicure Studio", "Beauty Parlor" (semantic match)

2. **Multilingual Support**
   - User: "Κόψιμο μαλλιών" (Greek)
   - Finds: Hair salons, barbershops (semantic similarity)

3. **Intent Understanding**
   - User: "I want to relax and unwind"
   - Finds: Spas, massage parlors, wellness centers

4. **Hybrid Search**
   - Combine semantic search (vector) with filters (location, type)
   - Best of both worlds

5. **RAG for Businesses**
   - Retrieve similar businesses from past successful bookings
   - "Users who liked Salon X also liked..."

## Implementation Checklist

- [ ] Create `business_embeddings` table in PostgreSQL
- [ ] Implement embedding generation for businesses
- [ ] Store embeddings when businesses are fetched
- [ ] Implement semantic search function
- [ ] Enhance agent to use semantic search
- [ ] Add sync mechanism (on-demand or background)
- [ ] Test semantic search with various queries
- [ ] Monitor embedding storage and performance

## Storage Estimates

- **Embedding size**: 1536 dimensions × 4 bytes = ~6KB per embedding
- **Per business**: 3 embeddings (name, description, full_text) = ~18KB
- **10,000 businesses**: ~180MB
- **Very manageable** for PostgreSQL

## Conclusion

**Recommended Approach**: Store only embeddings in PostgreSQL, keep full business data in MongoDB. This gives you:
- ✅ Semantic search capabilities
- ✅ No data duplication issues
- ✅ Single source of truth
- ✅ Minimal storage overhead
- ✅ Easy to implement and maintain

The agent can do semantic search to find relevant business_ids, then fetch full data from the Python backend API when needed.

