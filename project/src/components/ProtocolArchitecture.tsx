import { Lang } from '../translations';

interface Props {
  lang: Lang;
}

const initiative = {
  zh: {
    heading: '项目设计初衷与愿景',
    intro: '在当前的 Web3 与 Solana 生态中，缺乏针对散户投资者和诈骗受害者的有效保护及补偿机制。项目方撤池跑路（Rug Pull）后，受害者通常面临维权无门、损失无法挽回的困境。',
    bridge: 'α 协议的设立初衷，旨在通过去中心化的技术与经济模型，为受害者提供一个公开透明的互助与资产重组平台。',
    listTitle: '我们的核心目标包括：',
    items: [
      '机制化救助：以完全去中心化、无许可的技术架构替代一切人工审核流程。任何持有并质押 $α 的地址，皆自动获得周期性赔付池的比例分配资格，无需白名单，无需人工审批。',
      '自动化造血：将 Pump.fun 创作者换手费分成与绿标合规审计服务费 100% 以 USDC 形式即时归集至主分流账户，由底层合约按 50/20/20/10 规则无情拆分，零人工干预。',
      '纯粹的透明度：坚持不包装任何虚假额度、不注入虚拟数字，国库由 0 USDC 真实起跑，所有链上账本接受全体持有人随时监督与链上对账。',
    ],
  },
  en: {
    heading: 'Project Initiative & Vision',
    intro: 'In the current Solana and Web3 ecosystem, there is a severe lack of consumer protection and remediation mechanisms for retail investors and scam victims. Following project rug pulls, victims are routinely left without recourse or recovery options.',
    bridge: 'The α protocol was initiated to establish a fully decentralized, permissionless mutual aid and asset reorganization framework — code-enforced, auditable by any holder at any time.',
    listTitle: 'Our core objectives are:',
    items: [
      'Permissionless Relief: Replacing every manual review process with a fully decentralized, trustless architecture. Any address staking $α is automatically eligible for proportional epoch restitution — no whitelist, no admin approval required.',
      'Automated Revenue Generation: 100% of Pump.fun creator fees and project audit service fees are converted to USDC and instantly streamed to the master splitter account with zero manual intervention, then split 50/20/20/10 by immutable contract logic.',
      'Absolute Transparency: Rejecting any synthetic metric padding. The treasury starts strictly at 0 USDC, with all on-chain ledgers fully open to holder-driven programmatic audits at any time.',
    ],
  },
};

const architecture = {
  zh: [
    {
      title: '1. 核心造血引擎',
      subtitle: 'Core Revenue Stream',
      items: [
        '链上分红机制：接入符合合规开源协议的 Pump.fun 创作者换手交易费分成，100% 以 USDC 稳定币形式即时归集至主分流合约账户，无任何人工干预节点，无资金中转停留。',
        '商业服务营收：面向新项目的合规审计服务（清白绿标）收取固定服务费；面向被挂项目方申请平反的听证会收取高额技术听证费（若平反失败，该费用将直接单向路由至赔付池）。两类收入同样 100% 自动转换为 USDC 注入主分流账户。',
      ],
    },
    {
      title: '2. 国库四轨分配原则',
      subtitle: 'The Four-Fold Treasury Rule',
      description:
        '国库资金流水线实时拆分原则 (Real-Time Dynamic Split Router): 每一笔打入国库的 USDC 不计上限、无需等待，由底层代码无情卡死，按 5:2:2:1 比例即时机械分流到以下四轨:',
      items: [
        '50% [🏥 散户赔付救济金池]: 每期纪元(Epoch)国库累计的半数资金单向注入，由全网散户根据 ve-lock 质押算力按比例公平瓜分。',
        '20% [🔄 自动回购销毁池]: 智能路由自动执行。每当池内积攒满 200 USDC 立即调用 Jupiter API 扫货 $α 并打入黑洞，产生极高频的通缩买盘。',
        '20% [⚙️ 建设者运营工资池]: 阳光化分配。内部通过 4:3:2:1 比例保障技术、运营与陪审团开销。接收钱包地址槽完全由社区 DAO 投票(51%通过)动态锁定，随时可发起罢免或更换。',
        '10% [🥩 纯代币质押分红池]: 阳光普照奖。10% 的真实 USDC 现金流直接以秒级利息形式，按持币份额实时分发给所有 $α 锁仓死忠，打造真金白银的分红资产。',
      ],
    },
    {
      title: '3. 质押积分与防女巫设计',
      subtitle: 'Staking Points & Anti-Sybil Layout',
      items: [
        '算力逻辑绑定: 取消任何中心化白名单审核。用户的「救济金分配算力积分」完全取决于其在二级市场购买并投入质押的 $α 数量与连续锁仓时间的乘积加权：Points = 实际质押α数量 × 连续质押天数系数。大户若想瓜分更多 USDC，必须大额买入并长线锁仓，直接在前期为代币拉起高额买盘。',
        '熔断重置机制: 质押期间若发生任何解锁、转账或抛售行为，其「连续质押天数」计数器将立刻无条件重置为 0，彻底封杀大户投机性抢退的行业漏洞。',
        '5% 提现维税: 用户在提取 USDC 收益时自动扣除 5% 用于维持服务器运维及防分布式拒绝服务攻击(DDoS)开销。',
      ],
    },
    {
      title: '4. 核心风控防御协议',
      subtitle: 'Triple Security Patches',
      items: [
        '价格波动防御锚定：当二级市场面临剧烈波动时，系统自动激活价格保护模块，20% 回购销毁池加速清算，提示持有人当前通缩买盘效率的动态提升，引导社区将市场波动转化为锁仓共识。',
        'DAO 最高弹劾与动态调参机制: 社区不仅拥有对 20% 工资池各岗位负责人的一票否决与弹劾权；在项目运行后期(Epoch 10之后)，代币持有人更可通过 66% 绝对多数投票，动态微调 5:2:2:1 的初始国库分配比例，使协议具备自我进化的生命力。',
        '100% 链上真实宣言：坚持零注水原则，前端不包含任何虚设信用额度或虚拟基数，国库数据完全由 0 USDC 真实起跑，接受全网 Holder 对账。',
      ],
    },
  ],
  en: [
    {
      title: '1. Core Revenue Stream',
      subtitle: '',
      items: [
        "On-Chain Revenue Splitting: 100% of Pump.fun creator trading fees are converted to USDC and instantly streamed to the master splitter contract account with zero manual intervention — no transit stops, no human touchpoints.",
        'Commercial Gateways: Fixed USDC Audit Fees for compliance verification of new projects ("Green Label") and high-fixed Hearing Fees for listed developers applying for a re-trial (permanently single-routed into the compensation pool if the appeal fails). Both revenue types are likewise auto-converted 100% to USDC and injected directly into the master splitter.',
      ],
    },
    {
      title: '2. The Four-Fold Treasury Rule',
      subtitle: '',
      description:
        'Real-Time Dynamic Split Router: Every single USDC entering the treasury — with no cap, no waiting — is instantly and mechanically routed by immutable contract code into the following four tracks at a fixed 5:2:2:1 ratio:',
      items: [
        '50% [🏥 Retail Relief Pool]: Every epoch, 50% of all accumulated treasury inflows are injected one-way and distributed proportionally to all retail participants based on their ve-lock staking power.',
        '20% [🔄 Autonomous Buyback & Burn Pool]: Smart-routed automatically. Every time the pool accumulates 200 USDC, Jupiter API is called immediately to buy $α and send it to the null address — generating ultra-high-frequency deflationary buy pressure.',
        '20% [⚙️ Builder Operations Payroll Pool]: Fully transparent allocation. Internally split via a 4:3:2:1 rule covering tech, ops, and jury overhead. Recipient wallet address slots are dynamically locked by 51% community DAO vote and subject to impeachment at any time.',
        '10% [🥩 Pure Staking Dividend Pool]: The sunshine reward. 10% of real USDC cash flow is distributed in real-time as per-second interest directly to all $α long-term stakers — genuine yield, zero dilution.',
      ],
    },
    {
      title: '3. Staking Points & Anti-Sybil Layout',
      subtitle: '',
      items: [
        'Permissionless Power Binding: No centralized whitelist verification. A user\'s "Relief Distribution Power Points" are determined entirely by the $α purchased on the open market and staked, weighted by their consecutive lockup duration: Points = Actual Staked α × Consecutive Days Multiplier. Whales who want a larger USDC share must buy big and lock long — directly generating buy-side pressure on the token from day one.',
        'Melt Reset Mechanism: Any unlock, transfer, or sale during the staking period instantly and unconditionally resets the "Consecutive Staking Days" counter to 0 — permanently closing the speculative front-running loophole.',
        '5% Withdrawal Maintenance Tax: A 5% fee is automatically deducted on USDC reward withdrawals to cover server operations and DDoS protection overhead.',
      ],
    },
    {
      title: '4. Triple Security Patches',
      subtitle: '',
      items: [
        'Volatility Protection Anchor: Secondary market volatility automatically triggers structural optimizations — the 20% buyback pool accelerates burn velocity, alerting holders of dynamic deflationary efficiency improvements and converting market panic into locking consensus.',
        'Supreme DAO Impeachment & Dynamic Parameter Tuning: The community holds unilateral veto and impeachment rights over every 20% payroll pool role. Post-Epoch 10, token holders can further dynamically adjust the base 5:2:2:1 treasury allocation ratio via a 66% absolute supermajority vote — giving the protocol the capacity for self-evolution.',
        '100% On-Chain Verifiability: Complete rejection of synthetic metric padding or artificial baselines. The treasury starts strictly at 0 USDC, fully open to holder-driven programmatic audits at any time.',
      ],
    },
  ],
};

export default function ProtocolArchitecture({ lang }: Props) {
  const init = initiative[lang];
  const arch = architecture[lang];
  const archHeading =
    lang === 'zh'
      ? 'α 协议全盘构想与经济闭环'
      : 'α Protocol Architecture & Economic Loop';

  return (
    <div className="space-y-10">
      {/* Initiative & Vision */}
      <section className="border border-zinc-800 rounded-xl bg-zinc-900/50 backdrop-blur-sm p-6 space-y-4">
        <h3 className="text-lg font-bold text-zinc-200 font-mono tracking-wide border-b border-zinc-800 pb-3">
          {init.heading}
        </h3>
        <p className="text-xs text-zinc-400 font-mono leading-relaxed">
          {init.intro}
        </p>
        <p className="text-xs text-zinc-400 font-mono leading-relaxed">
          {init.bridge}
        </p>
        <p className="text-xs text-zinc-300 font-mono font-semibold mt-2">
          {init.listTitle}
        </p>
        <ol className="space-y-2 list-none">
          {init.items.map((item, i) => (
            <li
              key={i}
              className="text-xs text-zinc-400 font-mono leading-relaxed pl-3 border-l-2 border-zinc-700"
            >
              {item}
            </li>
          ))}
        </ol>
      </section>

      {/* Protocol Architecture */}
      <section className="space-y-6">
        <h3 className="text-lg font-bold text-zinc-200 font-mono tracking-wide border-b border-zinc-800 pb-3">
          {archHeading}
        </h3>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          {arch.map((section, i) => (
            <div
              key={i}
              className="border border-zinc-800 rounded-xl bg-zinc-900/50 backdrop-blur-sm p-5 space-y-3"
            >
              <div>
                <h4 className="text-sm font-bold text-zinc-100 font-mono">
                  {section.title}
                </h4>
                {section.subtitle && (
                  <p className="text-xs text-zinc-500 font-mono mt-0.5">
                    {section.subtitle}
                  </p>
                )}
              </div>

              {section.description && (
                <p className="text-xs text-zinc-300 font-mono leading-relaxed border-l-2 border-cyan-400/40 pl-3">
                  {section.description}
                </p>
              )}

              <ul className="space-y-2.5">
                {section.items.map((item, j) => (
                  <li
                    key={j}
                    className="text-xs text-zinc-400 font-mono leading-relaxed pl-3 border-l-2 border-zinc-700"
                  >
                    {item}
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
