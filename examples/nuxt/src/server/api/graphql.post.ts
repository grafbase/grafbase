export default defineEventHandler(async (event) => {
  const { GRAFBASE_API_URL, GRAFBASE_API_KEY } = process.env;
  const body = await useBody(event);

  if (!body || !GRAFBASE_API_URL || !GRAFBASE_API_KEY) {
    throw createError({
      statusCode: 500,
      message: `Missing required ${!body ? "body" : "environment variables"}.`,
    });
  }

  const response: any = await $fetch<any>(GRAFBASE_API_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${GRAFBASE_API_KEY}`,
    },
    body,
  });

  if (response.error) {
    throw createError({
      statusCode: 500,
      message: "Something went wrong.",
      cause: response.error,
    });
  }

  return response;
});
