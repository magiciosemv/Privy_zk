'use client'

import { useRef } from 'react'
import { motion, useInView } from 'framer-motion'
import Link from 'next/link'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { cn } from '@/lib/utils'
import {
  Shield,
  Code2,
  Eye,
  ArrowRight,
  ArrowDown,
  Zap,
  Terminal,
  Blocks,
  Fingerprint,
} from 'lucide-react'

const particles = Array.from({ length: 30 }, (_, i) => ({
  id: i,
  x: Math.random() * 100,
  y: Math.random() * 100,
  size: Math.random() * 4 + 2,
  duration: Math.random() * 6 + 4,
  delay: Math.random() * 3,
}))

const fadeInUp = {
  hidden: { opacity: 0, y: 40 },
  visible: { opacity: 1, y: 0 },
}

function ParticleField() {
  return (
    <div className="absolute inset-0 overflow-hidden pointer-events-none" aria-hidden="true">
      {particles.map((p) => (
        <motion.div
          key={p.id}
          className="absolute rounded-full bg-purple-400/30"
          style={{
            left: `${p.x}%`,
            top: `${p.y}%`,
            width: p.size,
            height: p.size,
          }}
          animate={{
            y: [0, -30, 0],
            opacity: [0.2, 0.6, 0.2],
            scale: [1, 1.5, 1],
          }}
          transition={{
            duration: p.duration,
            repeat: Infinity,
            delay: p.delay,
            ease: 'easeInOut',
          }}
        />
      ))}
    </div>
  )
}

function PulseRing() {
  return (
    <div className="absolute inset-0 flex items-center justify-center pointer-events-none" aria-hidden="true">
      <motion.div
        className="size-64 rounded-full border border-purple-500/20"
        animate={{ scale: [1, 1.3, 1], opacity: [0.15, 0.35, 0.15] }}
        transition={{ duration: 4, repeat: Infinity, ease: 'easeInOut' }}
      />
      <motion.div
        className="absolute size-48 rounded-full border border-indigo-400/20"
        animate={{ scale: [1, 1.5, 1], opacity: [0.1, 0.3, 0.1] }}
        transition={{ duration: 3.5, repeat: Infinity, ease: 'easeInOut', delay: 0.5 }}
      />
      <motion.div
        className="absolute size-32 rounded-full border border-violet-400/15"
        animate={{ scale: [1, 1.7, 1], opacity: [0.08, 0.25, 0.08] }}
        transition={{ duration: 3, repeat: Infinity, ease: 'easeInOut', delay: 1 }}
      />
    </div>
  )
}

function SectionInView({ children, className }: { children: React.ReactNode; className?: string }) {
  const ref = useRef(null)
  const isInView = useInView(ref, { once: true, margin: '-80px' })
  return (
    <motion.section
      ref={ref}
      initial={{ opacity: 0, y: 40 }}
      animate={isInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 40 }}
      transition={{ duration: 0.7, ease: [0.25, 0.46, 0.45, 0.94] }}
      className={className}
    >
      {children}
    </motion.section>
  )
}

const features = [
  {
    icon: Fingerprint,
    title: 'Field-Level Privacy',
    description: "One #[private] annotation per variable. Not all or nothing — keep public state public, private state private.",
    color: 'from-purple-500 to-purple-400',
  },
  {
    icon: Code2,
    title: 'Rust Native',
    description: 'Zero new language. Annotate, compile, deploy. No Noir required. Use the tools you already know.',
    color: 'from-indigo-500 to-indigo-400',
  },
  {
    icon: Eye,
    title: 'Selective Disclosure',
    description: 'Prove what you want, hide what you don\'t. GDPR-ready with granular attribute-level control.',
    color: 'from-violet-500 to-violet-400',
  },
]

const steps = [
  {
    num: 1,
    title: 'Annotate',
    icon: Terminal,
    code: '#[private]\n  balance: u64',
    desc: 'Add one annotation to any Rust variable',
  },
  {
    num: 2,
    title: 'Build',
    icon: Blocks,
    code: 'privy build\n  auto-ZK circuit',
    desc: 'Compiler auto-generates ZK circuits',
  },
  {
    num: 3,
    title: 'Deploy',
    icon: Zap,
    code: 'solana deploy\n  privacy enabled',
    desc: 'Deploy to Solana with privacy baked in',
  },
]

const demos = [
  {
    href: '/demos/poker',
    title: 'ZK Poker',
    desc: 'Hide your hand, prove your cards',
    icon: '♠',
    gradient: 'from-purple-600 to-pink-500',
  },
  {
    href: '/demos/darkpool',
    title: 'Privacy Dark Pool',
    desc: 'Trade without exposure',
    icon: '◆',
    gradient: 'from-indigo-600 to-blue-500',
  },
  {
    href: '/demos/vote',
    title: 'Anonymous Voting',
    desc: 'Vote privately, count publicly',
    icon: '✓',
    gradient: 'from-violet-600 to-cyan-500',
  },
]

const techStack = [
  'Solana', 'Anchor', 'Groth16', 'Plonk', 'Poseidon',
  'Next.js', 'Framer Motion', 'Rust', 'WASM', 'TypeScript',
]

export default function LandingPage() {
  const stepsRef = useRef(null)
  const stepsInView = useInView(stepsRef, { once: true, margin: '-80px' })

  const demosRef = useRef(null)
  const demosInView = useInView(demosRef, { once: true, margin: '-80px' })

  return (
    <div className="flex flex-col bg-neutral-950 text-neutral-100">
      {/* ─── Hero Section ─── */}
      <section className="relative min-h-screen flex flex-col items-center justify-center overflow-hidden">
        {/* Background gradient */}
        <div className="absolute inset-0 bg-gradient-to-b from-purple-950/60 via-neutral-950 to-neutral-950" aria-hidden="true" />
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,rgba(120,50,255,0.15),transparent_70%)]" aria-hidden="true" />

        <ParticleField />
        <PulseRing />

        <motion.div
          className="relative z-10 flex flex-col items-center gap-8 px-4 text-center"
          initial={{ opacity: 0, y: 30 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.8, ease: [0.25, 0.46, 0.45, 0.94] }}
        >
          {/* Badge */}
          <motion.div
            initial={{ opacity: 0, scale: 0.8 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={{ duration: 0.5, delay: 0.2 }}
          >
            <span className="inline-flex items-center gap-2 rounded-full border border-purple-500/30 bg-purple-500/10 px-4 py-1.5 text-sm text-purple-300">
              <Shield className="size-4" />
              Solana Hackathon 2026
            </span>
          </motion.div>

          {/* Title */}
          <motion.h1
            className="text-5xl font-extrabold tracking-tight sm:text-7xl md:text-8xl"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.3 }}
          >
            <span className="bg-gradient-to-r from-purple-400 via-indigo-300 to-violet-300 bg-clip-text text-transparent">
              Privy SVM
            </span>
          </motion.h1>

          {/* Subtitle */}
          <motion.p
            className="max-w-2xl text-lg text-neutral-300 sm:text-xl"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.5 }}
          >
            Programmable Privacy for Solana &mdash; one annotation away
          </motion.p>

          {/* Tagline */}
          <motion.div
            className="rounded-xl border border-neutral-800 bg-neutral-900/50 px-6 py-4 backdrop-blur-sm"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.7 }}
          >
            <code className="text-base text-neutral-200 sm:text-lg">
              <span className="text-purple-400">Write Rust.</span>{' '}
              <span className="text-indigo-400">Add</span>{' '}
              <span className="rounded bg-purple-500/20 px-1.5 py-0.5 font-mono text-purple-300">#[private]</span>
              <span className="text-indigo-400">.</span>{' '}
              <span className="text-violet-400">Get ZK privacy.</span>{' '}
              <span className="text-neutral-500">Zero learning curve.</span>
            </code>
          </motion.div>

          {/* CTA Buttons */}
          <motion.div
            className="flex flex-wrap items-center justify-center gap-4 pt-4"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.6, delay: 0.9 }}
          >
            <Link
              href="/demos"
              className="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-purple-600 to-indigo-600 px-6 py-3 text-sm font-semibold text-white shadow-lg shadow-purple-500/25 transition-transform hover:scale-105 active:scale-95"
            >
              Explore Demos
              <ArrowRight className="size-4" />
            </Link>
            <Link
              href="/docs"
              className="inline-flex items-center gap-2 rounded-xl border border-neutral-700 bg-neutral-800/50 px-6 py-3 text-sm font-semibold text-neutral-200 backdrop-blur-sm transition-transform hover:scale-105 hover:border-neutral-600 active:scale-95"
            >
              Read Docs
            </Link>
          </motion.div>
        </motion.div>

        {/* Scroll indicator */}
        <motion.div
          className="absolute bottom-8 flex flex-col items-center gap-2"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 1.5 }}
        >
          <span className="text-xs text-neutral-500">Scroll to explore</span>
          <motion.div
            animate={{ y: [0, 8, 0] }}
            transition={{ duration: 1.5, repeat: Infinity, ease: 'easeInOut' }}
          >
            <ArrowDown className="size-4 text-neutral-500" />
          </motion.div>
        </motion.div>
      </section>

      {/* ─── Innovation Section ─── */}
      <SectionInView className="px-4 py-24 sm:px-8">
        <div className="mx-auto flex max-w-6xl flex-col items-center gap-12">
          <div className="flex flex-col items-center gap-4 text-center">
            <Badge variant="outline" className="border-purple-500/30 text-purple-300">
              Why Privy SVM
            </Badge>
            <h2 className="text-3xl font-bold tracking-tight sm:text-4xl">
              Privacy at the{' '}
              <span className="bg-gradient-to-r from-purple-400 to-indigo-300 bg-clip-text text-transparent">
                Variable Level
              </span>
            </h2>
            <p className="max-w-2xl text-neutral-400">
              Not contract-level. Not transaction-level. Field-level — the granularity
              that real applications demand.
            </p>
          </div>

          <div className="grid gap-6 sm:grid-cols-3">
            {features.map((feature, i) => {
              const Icon = feature.icon
              return (
                <motion.div
                  key={feature.title}
                  initial="hidden"
                  whileInView="visible"
                  viewport={{ once: true, margin: '-60px' }}
                  variants={fadeInUp}
                  transition={{ duration: 0.6, delay: i * 0.15, ease: 'easeOut' }}
                >
                  <Card className="group relative border-neutral-800 bg-neutral-900/50 backdrop-blur-sm transition-all hover:scale-[1.03] hover:border-purple-500/30 hover:shadow-lg hover:shadow-purple-500/10">
                    <CardHeader>
                      <div
                        className={cn(
                          'mb-2 inline-flex size-12 items-center justify-center rounded-xl bg-gradient-to-br',
                          feature.color,
                          'bg-opacity-10 p-2.5'
                        )}
                      >
                        <Icon className="size-6 text-white" />
                      </div>
                      <CardTitle className="text-lg text-neutral-100">{feature.title}</CardTitle>
                    </CardHeader>
                    <CardContent>
                      <CardDescription className="text-neutral-400">{feature.description}</CardDescription>
                    </CardContent>
                  </Card>
                </motion.div>
              )
            })}
          </div>
        </div>
      </SectionInView>

      {/* ─── How It Works Section ─── */}
      <SectionInView className="px-4 py-24 sm:px-8">
        <div className="mx-auto flex max-w-5xl flex-col items-center gap-12">
          <div className="flex flex-col items-center gap-4 text-center">
            <Badge variant="outline" className="border-indigo-500/30 text-indigo-300">
              How It Works
            </Badge>
            <h2 className="text-3xl font-bold tracking-tight sm:text-4xl">
              Three steps to{' '}
              <span className="bg-gradient-to-r from-indigo-400 to-violet-300 bg-clip-text text-transparent">
                privacy
              </span>
            </h2>
          </div>

          <div
            ref={stepsRef}
            className="flex w-full flex-col items-center gap-4 md:flex-row md:items-start"
          >
            {steps.map((step, i) => {
              const Icon = step.icon
              return (
                <div key={step.title} className="flex flex-1 flex-col items-center gap-0 md:flex-row">
                  <motion.div
                    className="flex w-full flex-col items-center gap-4 text-center"
                    initial={{ opacity: 0, y: 30 }}
                    animate={stepsInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 30 }}
                    transition={{ duration: 0.5, delay: i * 0.2, ease: [0.25, 0.46, 0.45, 0.94] }}
                  >
                    {/* Step number circle */}
                    <div className="flex size-14 items-center justify-center rounded-full bg-gradient-to-br from-purple-600 to-indigo-600 shadow-lg shadow-purple-500/20">
                      <span className="text-xl font-bold text-white">{step.num}</span>
                    </div>

                    <h3 className="flex items-center gap-2 text-lg font-semibold text-neutral-100">
                      <Icon className="size-5 text-purple-400" />
                      {step.title}
                    </h3>

                    <div className="rounded-lg border border-neutral-800 bg-neutral-900/70 px-6 py-4 backdrop-blur-sm">
                      <code className="text-sm leading-relaxed whitespace-pre text-neutral-300">
                        {step.code}
                      </code>
                    </div>

                    <p className="text-sm text-neutral-500">{step.desc}</p>
                  </motion.div>

                  {/* Arrow between steps */}
                  {i < steps.length - 1 && (
                    <div className="flex items-center justify-center py-4 md:px-2 md:py-0 md:pt-10">
                      <ArrowRight className="hidden size-6 text-neutral-600 md:block" />
                      <ArrowDown className="size-6 text-neutral-600 md:hidden" />
                    </div>
                  )}
                </div>
              )
            })}
          </div>
        </div>
      </SectionInView>

      {/* ─── Demo Showcase Section ─── */}
      <SectionInView className="px-4 py-24 sm:px-8">
        <div className="mx-auto flex max-w-5xl flex-col items-center gap-12">
          <div className="flex flex-col items-center gap-4 text-center">
            <Badge variant="outline" className="border-violet-500/30 text-violet-300">
              Interactive Demos
            </Badge>
            <h2 className="text-3xl font-bold tracking-tight sm:text-4xl">
              See privacy{' '}
              <span className="bg-gradient-to-r from-violet-400 to-purple-300 bg-clip-text text-transparent">
                in action
              </span>
            </h2>
            <p className="max-w-xl text-neutral-400">
              Three fully interactive demos showcasing what Privy SVM makes possible.
            </p>
          </div>

          <div
            ref={demosRef}
            className="grid w-full gap-6 sm:grid-cols-3"
          >
            {demos.map((demo, i) => (
              <motion.div
                key={demo.title}
                initial={{ opacity: 0, y: 30 }}
                animate={demosInView ? { opacity: 1, y: 0 } : { opacity: 0, y: 30 }}
                transition={{ duration: 0.5, delay: i * 0.15, ease: [0.25, 0.46, 0.45, 0.94] }}
              >
                <Link href={demo.href} className="group block h-full">
                  <div className="relative h-full overflow-hidden rounded-2xl border border-neutral-800 bg-neutral-900/60 p-[1px] transition-all group-hover:shadow-lg group-hover:shadow-purple-500/10">
                    {/* Gradient border overlay */}
                    <div
                      className={cn(
                        'absolute inset-0 rounded-2xl opacity-0 transition-opacity duration-500 group-hover:opacity-100',
                        'bg-gradient-to-r',
                        demo.gradient
                      )}
                      aria-hidden="true"
                    />
                    <div className="relative flex h-full flex-col items-center gap-4 rounded-2xl bg-neutral-950 p-8 text-center">
                      <div className="flex size-16 items-center justify-center rounded-2xl bg-neutral-800/50 text-3xl transition-transform group-hover:scale-110">
                        {demo.icon}
                      </div>
                      <h3 className="text-xl font-semibold text-neutral-100">{demo.title}</h3>
                      <p className="text-sm text-neutral-400">{demo.desc}</p>
                      <span className="mt-auto inline-flex items-center gap-1 text-sm font-medium text-purple-400 transition-colors group-hover:text-purple-300">
                        Launch demo
                        <ArrowRight className="size-3.5 transition-transform group-hover:translate-x-1" />
                      </span>
                    </div>
                  </div>
                </Link>
              </motion.div>
            ))}
          </div>
        </div>
      </SectionInView>

      {/* ─── Tech Stack Footer ─── */}
      <footer className="border-t border-neutral-800 px-4 py-16 sm:px-8">
        <div className="mx-auto flex max-w-4xl flex-col items-center gap-8">
          <div className="flex flex-col items-center gap-2">
            <div className="flex items-center gap-2">
              <Shield className="size-5 text-purple-400" />
              <span className="text-lg font-semibold text-neutral-200">Privy SVM</span>
            </div>
            <p className="text-sm text-neutral-500">
              Programmable privacy for the Solana ecosystem
            </p>
          </div>

          <div className="flex flex-wrap items-center justify-center gap-2">
            {techStack.map((tech) => (
              <Badge
                key={tech}
                variant="outline"
                className="border-neutral-700 bg-neutral-800/40 text-neutral-400 transition-colors hover:border-purple-500/40 hover:text-purple-300"
              >
                {tech}
              </Badge>
            ))}
          </div>

          <p className="text-xs text-neutral-600">
            Built for the Solana Hackathon &middot; {new Date().getFullYear()}
          </p>
        </div>
      </footer>
    </div>
  )
}
