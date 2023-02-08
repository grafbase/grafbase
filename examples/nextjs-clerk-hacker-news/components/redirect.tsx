import { useRouter } from "next/router";
import { PropsWithChildren, useEffect } from "react";

const Redirect = ({ children }: PropsWithChildren) => {
  const { push } = useRouter();

  useEffect(() => {
    push("/login");
  }, [push]);

  return <>{children}</>;
};

export default Redirect;
