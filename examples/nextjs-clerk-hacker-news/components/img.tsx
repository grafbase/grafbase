import { HTMLProps } from "react";

const Img = (
  props: Omit<HTMLProps<HTMLImageElement>, "src"> & {
    src: string | null | undefined;
  }
) => {
  return (
    // @ts-ignore
    <img
      alt={props.alt || "Image"}
      onError={(i) => (i.currentTarget.src = "/avatar.svg")}
      {...props}
    />
  );
};

export default Img;
