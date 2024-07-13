import Image from "next/image";
import Link from "next/link";
export default function Home() {
  return (
    <main className="flex flex-col items-center p-24">
      <h1 className="text-[120px] font-[500]">DataZen</h1>
      <p className="text-lg font-normal">
        Monetize computation on data without worrying about Data Leaks !!
      </p>
      <div className="grid grid-flow-col w-screen px-32 mt-[100px] justify-center space-x-[240px]">
        <Link
          className="border border-[#A5D6FF] rounded-md min-h-[200px] w-[40vh] flex flex-col items-center justify-center text-wrap p-5 text-center cursor-pointer bg-[#d0f1ff] hover:border-[#11023B]"
          href={"/store"}
        >
          <h1 className="text-[40px] font-light">Store</h1>
          <p className="font-extralight">
            Encrypt Data on client-side and enable compute on data
          </p>
        </Link>
        <Link
          className="border border-[#A5D6FF] rounded-md min-h-[200px] w-[40vh] flex flex-col items-center justify-center text-wrap p-5 text-center cursor-pointer bg-[#d0f1ff] hover:border-[#11023B]"
          href={"/compute"}
        >
          <h1 className="text-[40px] font-light">Compute</h1>
          <p className="font-extralight">
            Perform compute on high quality, tamper free data
          </p>
        </Link>
      </div>
    </main>
  );
}
