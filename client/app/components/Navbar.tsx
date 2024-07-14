"use client";
import React from "react";
import Image from "next/image";
import Link from "next/link";
import {
  useAccountModal,
  useChainModal,
  useConnectModal,
} from "@rainbow-me/rainbowkit";
import { useAccount, useDisconnect } from "wagmi";
const Navbar = () => {
  const { openConnectModal } = useConnectModal();
  const { address, isConnected } = useAccount();
  const { disconnect } = useDisconnect();
  return (
    <div className="flex flex-row items-center bg-inherit justify-between px-[40px] pt-[30px]">
      <Link className="border border-[#11023B] rounded-md bg-[#63B9FF] p-2 items-center w-[35px] h-[35px]" href={"/"}>
        <Image src={"/home.svg"} width={19} height={19} alt="disconnect" />
      </Link>
      <div className="flex flex-row items-center justify-center space-x-[40px]">
        {" "}
        <div className="flex font-[400]">
          <Link href={"https://abhays-organization-1.gitbook.io/datazen-docs/"} target="_blank">Docs</Link>
        </div>
        {isConnected ? (
          <div className="flex space-x-5 items-center">
            <button className="flex space-x-3 border p-2 border-[#11023B] rounded-md bg-[#DEF1FF]">
              <span>{`${address!.slice(0, 6)}...${address!.slice(
                -5,
                -1
              )}`}</span>
              <Image src={"/wallet.svg"} width={23} height={19} alt="wallet" />
            </button>
            <button
              className="border border-[#11023B] rounded-md bg-[#63B9FF] p-2 items-center w-[35px] h-[35px]"
              onClick={() => disconnect()}
            >
              <Image
                src={"/disconnect.svg"}
                width={19}
                height={19}
                alt="disconnect"
              />
            </button>
          </div>
        ) : (
          <>
            {" "}
            {openConnectModal && (
              <div onClick={openConnectModal}>
                <button className="flex space-x-3 border p-2 border-[#11023B] rounded-md bg-[#DEF1FF] cursor-pointer">
                  <span>Wallet Connect</span>
                  <Image
                    src={"/wallet.svg"}
                    width={23}
                    height={19}
                    alt="wallet"
                  />
                </button>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
};

export default Navbar;
