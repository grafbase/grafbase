import type { NextApiRequest, NextApiResponse } from "next";
import serverSideFetch from "utils/server-side-fetch";

type Data = {
  data?: string;
};

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse<Data>
) {
  res.status(200).json(await serverSideFetch(req.body));
}
