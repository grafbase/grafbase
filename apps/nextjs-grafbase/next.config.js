const { withGrafbase } = require('@grafbase/nextjs-plugin')

/** @type {import('next').NextConfig} */
const nextConfig = {}

module.exports = withGrafbase(nextConfig)
