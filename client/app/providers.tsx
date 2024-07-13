// app/providers.tsx
"use client";

import { NextUIProvider } from "@nextui-org/react";
import "@rainbow-me/rainbowkit/styles.css";
import {
  getDefaultConfig,
  lightTheme,
  RainbowKitProvider,
  Theme,
} from "@rainbow-me/rainbowkit";
import { WagmiProvider } from "wagmi";
import { filecoinCalibration, polygonZkEvm, arbitrumSepolia } from "wagmi/chains";
import { QueryClientProvider, QueryClient } from "@tanstack/react-query";

const config = getDefaultConfig({
  appName: "Gasdaq",
  projectId: "db1b8a46ffa835bd9a48a89ff540f990",
  chains: [filecoinCalibration, polygonZkEvm, arbitrumSepolia],
  ssr: true,
});

const queryClient = new QueryClient();

const customTheme = {
  colors: {
    accentColor: "hsl(0 0% 0%)",
    accentColorForeground: "hsl(0, 0%, 100%)",
    actionButtonBorder: "hsl(206 100% 93%)",
    actionButtonBorderMobile: "hsl(206 100% 93%)",
    actionButtonSecondaryBackground: "hsl(0, 0%, 100%)",
    closeButton: "hsl(206 3% 59%)",
    closeButtonBackground: "hsl(206 31% 83%)",
    connectButtonBackground: "hsl(206 100% 93%)",
    connectButtonBackgroundError: "hsl(0 100% 64%)",
    connectButtonInnerBackground: "hsl(206 51% 88%)",
    connectButtonText: "hsl(0, 0%, 100%)",
    connectButtonTextError: "hsl(0, 0%, 100%)",
    error: "hsl(0, 0%, 100%)",
    generalBorder: "hsl(180, 0%, 94%)",
    generalBorderDim: "rgba(0, 0, 0, 0.03)",
    menuItemBackground: "hsl(206 51% 88%)",
    modalBackdrop: "rgba(0, 0, 0, 0.5)",
    modalBackground: "hsl(206 100% 93%)",
    modalBorder: "hsl(180, 0%, 94%)",
    modalText: "hsl(225, 0%, 0%)",
    modalTextDim: "rgba(60, 66, 66, 0.3)",
    modalTextSecondary: "hsl(200,1%,55%)",
    profileAction: "hsl(206 51% 88%)",
    profileActionHover: "hsl(206 31% 83%)",
    profileForeground: "hsl(206 100% 93%)",
    selectedOptionBorder: "hsl(0 0% 0%)",
    downloadBottomCardBackground:
      "linear-gradient(126deg, rgba(255, 255, 255, 0) 9.49%, rgba(171, 171, 171, 0.04) 71.04%), #FFFFFF",
    downloadTopCardBackground:
      "linear-gradient(126deg, rgba(171, 171, 171, 0.2) 9.49%, rgba(255, 255, 255, 0) 71.04%), #FFFFFF",
    connectionIndicator: "hsl(107, 100%, 44%)",
    standby: "hsl(47, 100%, 63%)",
  },
  radii: {
    actionButton: "6px",
    connectButton: "3px",
    menuButton: "3px",
    modal: "6px",
    modalMobile: "6px",
  },
  shadows: {
    connectButton: "0px 8px 32px rgba(0,0,0,.32)",
    dialog: "0px 8px 32px rgba(0,0,0,.32)",
    profileDetailsAction: "0px 2px 6px rgba(37, 41, 46, 0.04)",
    selectedOption: "0px 2px 6px rgba(0, 0, 0, 0.24)",
    selectedWallet: "0px 2px 6px rgba(0, 0, 0, 0.12)",
    walletLogo: "0px 2px 16px rgba(0, 0, 0, 0.16)",
  },
  blurs: {
    modalOverlay: "blur(0px)", // e.g. 'blur(4px)'
  },
  fonts: {
    body: "...", // default
  },
};

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <NextUIProvider>
      <WagmiProvider config={config}>
        <QueryClientProvider client={queryClient}>
          <RainbowKitProvider theme={customTheme} modalSize="compact">
            {children}{" "}
          </RainbowKitProvider>
        </QueryClientProvider>
      </WagmiProvider>
    </NextUIProvider>
  );
}