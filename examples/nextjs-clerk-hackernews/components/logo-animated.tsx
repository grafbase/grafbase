import { SVGProps } from "react";

const LogoAnimated = (props: SVGProps<SVGSVGElement>) => {
  return (
    <svg
      version="1.1"
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 28"
      className="w-24 h-24"
      {...props}
    >
      <polyline
        fill="none"
        stroke="currentColor"
        strokeWidth={2}
        strokeLinecap="round"
        strokeLinejoin="round"
        points="12,1 1,6.5 12,12 23,6.5 17.5,4 12,6.5 "
        className="logo-first-line"
      />
      <polyline
        fill="none"
        stroke="currentColor"
        strokeWidth={2}
        strokeLinecap="round"
        strokeLinejoin="round"
        className="logo-second-line"
        points="1,12 12,17.5 23,12 "
      />
      <polyline
        fill="none"
        stroke="currentColor"
        strokeWidth={2}
        strokeLinecap="round"
        strokeLinejoin="round"
        points="1,17.5 12,23 23,17.5 "
        className="logo-third-line"
      />
    </svg>
  );
};

export default LogoAnimated;
