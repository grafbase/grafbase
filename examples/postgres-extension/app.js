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
  },
  inventory: {
    sku: null,
    variantName: '',
    data: null,
    pageInfo: null,
    currentPage: 1,
    loading: false,
  },
}

// DOM Elements
const elements = {
  productsSection: document.getElementById('products-section'),
  variantsSection: document.getElementById('variants-section'),
  inventorySection: document.getElementById('inventory-section'),

  productsLoading: document.getElementById('products-loading'),
  variantsLoading: document.getElementById('variants-loading'),
  inventoryLoading: document.getElementById('inventory-loading'),

  productsBody: document.getElementById('products-body'),
  variantsBody: document.getElementById('variants-body'),
  inventoryBody: document.getElementById('inventory-body'),

  productsPrev: document.getElementById('products-prev'),
  productsNext: document.getElementById('products-next'),
  variantsPrev: document.getElementById('variants-prev'),
  variantsNext: document.getElementById('variants-next'),
  inventoryPrev: document.getElementById('inventory-prev'),
  inventoryNext: document.getElementById('inventory-next'),

  productsPageInfo: document.getElementById('products-page-info'),
  variantsPageInfo: document.getElementById('variants-page-info'),
  inventoryPageInfo: document.getElementById('inventory-page-info'),

  productName: document.getElementById('product-name'),
  variantName: document.getElementById('variant-name'),

  backToProducts: document.getElementById('back-to-products'),
  backToVariants: document.getElementById('back-to-variants'),
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

const INVENTORY_QUERY = `
  query GetInventory($sku: String, $first: Int, $after: String, $before: String, $last: Int) {
    inventoryInventories(
      filter: {
        sku: { eq: $sku }
      },
      first: $first,
      after: $after,
      before: $before,
      last: $last
    ) {
      edges {
        node {
          id
          sku
          quantity
          warehouseLocation
          updatedAt
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

async function fetchInventory(variantName, sku, forward = true, cursor = null) {
  if (sku) {
    state.inventory.sku = sku
    state.inventory.variantName = variantName
  }

  state.inventory.loading = true
  elements.inventoryLoading.classList.remove('hidden')
  updateInventoryUI()

  const variables = {
    sku: state.inventory.sku,
    ...(forward ? { first: ITEMS_PER_PAGE, after: cursor } : { last: ITEMS_PER_PAGE, before: cursor }),
  }

  const result = await fetchGraphQL(INVENTORY_QUERY, variables)

  if (result && result.data) {
    state.inventory.data = result.data.inventoryInventories.edges.map((edge) => ({
      ...edge.node,
      cursor: edge.cursor,
    }))
    state.inventory.pageInfo = result.data.inventoryInventories.pageInfo
    state.inventory.currentPage =
      forward && cursor
        ? state.inventory.currentPage + 1
        : !forward && cursor
          ? state.inventory.currentPage - 1
          : state.inventory.currentPage
  }

  state.inventory.loading = false
  elements.inventoryLoading.classList.add('hidden')
  updateInventoryUI()
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
      <td>
        <button class="btn-action" data-variant-name="${variant.name || 'Variant'}" data-variant-sku="${variant.sku}">
          View Inventory
        </button>
      </td>
    `
    elements.variantsBody.appendChild(row)
  })

  // Add event listeners for the "View Inventory" buttons
  document.querySelectorAll('[data-variant-sku]').forEach((button) => {
    button.addEventListener('click', (e) => {
      const variantName = e.target.dataset.variantName
      const sku = e.target.dataset.variantSku
      viewInventory(variantName, sku)
    })
  })
}

function renderInventory() {
  elements.inventoryBody.innerHTML = ''

  if (!state.inventory.data) return

  state.inventory.data.forEach((inventory) => {
    const row = document.createElement('tr')
    row.innerHTML = `
      <td>${inventory.id.substr(0, 8)}...</td>
      <td>${inventory.sku}</td>
      <td>${inventory.quantity}</td>
      <td>${inventory.warehouseLocation || 'N/A'}</td>
      <td>${formatDateTime(inventory.updatedAt)}</td>
    `
    elements.inventoryBody.appendChild(row)
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

function updateInventoryUI() {
  renderInventory()

  // Update title and pagination
  elements.variantName.textContent = state.inventory.variantName
  elements.inventoryPrev.disabled = !state.inventory.pageInfo?.hasPreviousPage
  elements.inventoryNext.disabled = !state.inventory.pageInfo?.hasNextPage
  elements.inventoryPageInfo.textContent = `Page ${state.inventory.currentPage}`
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

function viewInventory(variantName, sku) {
  state.inventory = {
    sku: null,
    variantName: '',
    data: null,
    pageInfo: null,
    currentPage: 1,
    loading: false,
  }

  elements.variantsSection.classList.add('hidden')
  elements.inventorySection.classList.remove('hidden')

  fetchInventory(variantName, sku)
}

function backToProducts() {
  elements.variantsSection.classList.add('hidden')
  elements.productsSection.classList.remove('hidden')
}

function backToVariants() {
  elements.inventorySection.classList.add('hidden')
  elements.variantsSection.classList.remove('hidden')
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

elements.inventoryPrev.addEventListener('click', () => {
  if (state.inventory.pageInfo.hasPreviousPage) {
    fetchInventory(null, null, false, state.inventory.pageInfo.startCursor)
  }
})

elements.inventoryNext.addEventListener('click', () => {
  if (state.inventory.pageInfo.hasNextPage) {
    fetchInventory(null, null, true, state.inventory.pageInfo.endCursor)
  }
})

elements.backToProducts.addEventListener('click', backToProducts)
elements.backToVariants.addEventListener('click', backToVariants)

// Initialize the application
async function init() {
  await fetchProducts()
}

// Start the application
document.addEventListener('DOMContentLoaded', init)
