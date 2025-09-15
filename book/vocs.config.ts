import { defineConfig } from "vocs";

export default defineConfig({
  title: "Hybrid Ethereum",
  description: "A comprehensive framework and node for developing, deploying, and interacting with smart contracts written in Rust and Solidity (including any language that compiles to EVM bytecode, such as Vyper, Yul, Huff, etc.)",
  sidebar: [
    {
      text: "Developers",
      items: [
        {
          text: "Overview",
          link: "/developers/overview",
        },
        {
          text: "Installation",
          link: "/developers/installation",
        },
        {
          text: "Quickstart",
          link: "/developers/quickstart",
        },
      ],
    },
    {
      text: "Examples",
      items: [
        {
          text: "ERC20",
          link: "/developers/examples/erc20",
        },
        {
          text: "Storage",
          link: "/developers/examples/storage",
        },
      ],
    },
    {
      text: "Protocols",
      items: [
        {
          text: "Overview",
          link: "/protocols/overview",
        },
        {
          text: "EVM Interpreter",
          items: [
            {
              text: "Overview",
              link: "/protocols/mini-evm-interpreter",
            },
          ],
        },
      ],
    },
  ],
  topNav: [
    { 
      text: 'Developers',
      link: '/developers/quickstart',
    },
    {
      text: 'Protocols',
      link: '/protocols/overview',
    },
  ],
  socials: [
    {
      icon: "github",
      link: "https://github.com/developeruche/hybrid",
    },
    {
      icon: "telegram",
      link: "https://t.me/developeruche",
    },
    {
      icon: "x",
      link: "https://x.com/developeruche",
    },
  ],
});
