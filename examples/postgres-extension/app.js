// Configuration
const API_URL = 'http://localhost:5000/graphql' // Update with your Grafbase GraphQL endpoint
const ITEMS_PER_PAGE = 5 // Number of items to display per page

// State management
const state = {
  products: {
    data: null,
    pageInfo: null,
    currentPage: 1,
    loading: false,
  },
  variants: {
    productId: null,
    productName: '',
    data: null,
    pageInfo: null,
    currentPage: 1,
    loading: false,
  }
}

// DOM Elements
const elements = {
  productsSection: document.getElementById('products-section'),
  variantsSection: document.getElementById('variants-section'),

  productsLoading: document.getElementById('products-loading'),
  variantsLoading: document.getElementById('variants-loading'),

  productsBody: document.getElementById('products-body'),
  variantsBody: document.getElementById('variants-body'),

  productsPrev: document.getElementById('products-prev'),
  productsNext: document.getElementById('products-next'),
  variantsPrev: document.getElementById('variants-prev'),
  variantsNext: document.getElementById('variants-next'),

  productsPageInfo: document.getElementById('products-page-info'),
  variantsPageInfo: document.getElementById('variants-page-info'),

  productName: document.getElementById('product-name'),

  backToProducts: document.getElementById('back-to-products'),
}

// GraphQL Queries
const PRODUCTS_QUERY = `
  query GetProducts($first: Int, $after: String, $before: String, $last: Int) {
    productsProducts(
      first: $first,
      after: $after,
      before: $before,
      last: $last,
      orderBy: [{ name: ASC }]
    ) {
      edges {
        node {
          id
          sku
          name
          slug
          price
        }
        cursor
      }
      pageInfo {
        hasNextPage
        hasPreviousPage
        startCursor
        endCursor
      }
    }
  }
`

const VARIANTS_QUERY = `
  query GetVariants($productId: UUID!, $first: Int, $after: String, $before: String, $last: Int) {
    productsProduct(lookup: { id: $productId }) {
      id
      name
      variants(
        first: $first,
        after: $after,
        before: $before,
        last: $last
      ) {
        edges {
          node {
            id
            sku
            name
            price
            inventory {
              quantity
              warehouseLocation
              updatedAt
            }
          }
          cursor
        }
        pageInfo {
          hasNextPage
          hasPreviousPage
          startCursor
          endCursor
        }
      }
    }
  }
`



function formatDateTime(timestamp) {
  return new Date(timestamp).toLocaleString()
}

async function fetchGraphQL(query, variables) {
  try {
    const response = await fetch(API_URL, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        query,
        variables,
      }),
    })

    if (!response.ok) {
      throw new Error(`Network error: ${response.status} ${response.statusText}`)
    }

    return await response.json()
  } catch (error) {
    console.error('Error fetching data:', error)
    return null
  }
}

// Data Fetching Functions
async function fetchProducts(forward = true, cursor = null) {
  state.products.loading = true
  elements.productsLoading.classList.remove('hidden')
  updateProductsUI()

  const variables = forward ? { first: ITEMS_PER_PAGE, after: cursor } : { last: ITEMS_PER_PAGE, before: cursor }

  const result = await fetchGraphQL(PRODUCTS_QUERY, variables)

  if (result && result.data) {
    state.products.data = result.data.productsProducts.edges.map((edge) => ({
      ...edge.node,
      cursor: edge.cursor,
    }))
    state.products.pageInfo = result.data.productsProducts.pageInfo
    state.products.currentPage =
      forward && cursor
        ? state.products.currentPage + 1
        : !forward && cursor
          ? state.products.currentPage - 1
          : state.products.currentPage
  }

  state.products.loading = false
  elements.productsLoading.classList.add('hidden')
  updateProductsUI()
}

async function fetchVariants(productId, productName, forward = true, cursor = null) {
  if (productId) {
    state.variants.productId = productId
    state.variants.productName = productName
  }

  state.variants.loading = true
  elements.variantsLoading.classList.remove('hidden')
  updateVariantsUI()

  const variables = {
    productId: state.variants.productId,
    ...(forward ? { first: ITEMS_PER_PAGE, after: cursor } : { last: ITEMS_PER_PAGE, before: cursor }),
  }

  const result = await fetchGraphQL(VARIANTS_QUERY, variables)

  if (result && result.data && result.data.productsProduct) {
    const variantsData = result.data.productsProduct.variants
    state.variants.data = variantsData.edges.map((edge) => ({
      ...edge.node,
      cursor: edge.cursor,
    }))
    state.variants.pageInfo = variantsData.pageInfo
    state.variants.currentPage =
      forward && cursor
        ? state.variants.currentPage + 1
        : !forward && cursor
          ? state.variants.currentPage - 1
          : state.variants.currentPage
  }

  state.variants.loading = false
  elements.variantsLoading.classList.add('hidden')
  updateVariantsUI()
}



// UI Rendering Functions
function renderProducts() {
  elements.productsBody.innerHTML = ''

  if (!state.products.data) return

  state.products.data.forEach((product) => {
    const row = document.createElement('tr')
    row.innerHTML = `
      <td>${product.id.substr(0, 8)}...</td>
      <td>${product.sku}</td>
      <td>${product.name}</td>
      <td>${product.price}</td>
      <td>
        <button class="btn-action" data-product-id="${product.id}" data-product-name="${product.name}">
          View Variants
        </button>
      </td>
    `
    elements.productsBody.appendChild(row)
  })

  // Add event listeners for the "View Variants" buttons
  document.querySelectorAll('[data-product-id]').forEach((button) => {
    button.addEventListener('click', (e) => {
      const productId = e.target.dataset.productId
      const productName = e.target.dataset.productName
      viewVariants(productId, productName)
    })
  })
}

function renderVariants() {
  elements.variantsBody.innerHTML = ''

  if (!state.variants.data) return

  state.variants.data.forEach((variant) => {
    const row = document.createElement('tr')
    row.innerHTML = `
      <td>${variant.id.substr(0, 8)}...</td>
      <td>${variant.sku}</td>
      <td>${variant.name || 'N/A'}</td>
      <td>${variant.price ? variant.price : 'Same as product'}</td>
      <td>${variant.inventory ? variant.inventory.quantity : 'N/A'}</td>
      <td>${variant.inventory ? (variant.inventory.warehouseLocation || 'N/A') : 'N/A'}</td>
      <td>${variant.inventory ? formatDateTime(variant.inventory.updatedAt) : 'N/A'}</td>
    `
    elements.variantsBody.appendChild(row)
  })
}



// UI Update Functions
function updateProductsUI() {
  renderProducts()

  // Update pagination buttons
  elements.productsPrev.disabled = !state.products.pageInfo?.hasPreviousPage
  elements.productsNext.disabled = !state.products.pageInfo?.hasNextPage
  elements.productsPageInfo.textContent = `Page ${state.products.currentPage}`
}

function updateVariantsUI() {
  renderVariants()

  // Update title and pagination
  elements.productName.textContent = state.variants.productName
  elements.variantsPrev.disabled = !state.variants.pageInfo?.hasPreviousPage
  elements.variantsNext.disabled = !state.variants.pageInfo?.hasNextPage
  elements.variantsPageInfo.textContent = `Page ${state.variants.currentPage}`
}

// Navigation Functions
function viewVariants(productId, productName) {
  state.variants = {
    productId: null,
    productName: '',
    data: null,
    pageInfo: null,
    currentPage: 1,
    loading: false,
  }

  elements.productsSection.classList.add('hidden')
  elements.variantsSection.classList.remove('hidden')

  fetchVariants(productId, productName)
}

function backToProducts() {
  elements.variantsSection.classList.add('hidden')
  elements.productsSection.classList.remove('hidden')
}

// Event Listeners
elements.productsPrev.addEventListener('click', () => {
  if (state.products.pageInfo.hasPreviousPage) {
    fetchProducts(false, state.products.pageInfo.startCursor)
  }
})

elements.productsNext.addEventListener('click', () => {
  if (state.products.pageInfo.hasNextPage) {
    fetchProducts(true, state.products.pageInfo.endCursor)
  }
})

elements.variantsPrev.addEventListener('click', () => {
  if (state.variants.pageInfo.hasPreviousPage) {
    fetchVariants(null, null, false, state.variants.pageInfo.startCursor)
  }
})

elements.variantsNext.addEventListener('click', () => {
  if (state.variants.pageInfo.hasNextPage) {
    fetchVariants(null, null, true, state.variants.pageInfo.endCursor)
  }
})

elements.backToProducts.addEventListener('click', backToProducts)

// Initialize the application
async function init() {
  await fetchProducts()
}

// Start the application
document.addEventListener('DOMContentLoaded', init)
