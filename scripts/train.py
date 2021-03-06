# pylint: disable=not-callable, no-member, invalid-name, missing-docstring, arguments-differ
import argparse
import itertools
import os

import torch
import torch.nn as nn
import tqdm

import time_logging
from hanabi import Game


def mean(xs):
    xs = list(xs)
    return sum(xs) / len(xs)


@torch.jit.script
def swish_jit_fwd(x):
    return x * torch.sigmoid(x) * 1.6768


@torch.jit.script
def swish_jit_bwd(x, grad_output):
    x_sigmoid = torch.sigmoid(x)
    return grad_output * (x_sigmoid * (1 + x * (1 - x_sigmoid))) * 1.6768


class SwishJitAutoFn(torch.autograd.Function):
    @staticmethod
    def forward(ctx, x):
        ctx.save_for_backward(x)
        return swish_jit_fwd(x)

    @staticmethod
    def backward(ctx, grad_output):
        x = ctx.saved_tensors[0]
        return swish_jit_bwd(x, grad_output)


class Swish(nn.Module):
    def forward(self, x):
        return SwishJitAutoFn.apply(x)


def orthogonal_(tensor, gain=1):
    '''
    Orthogonal initialization (modified version from PyTorch)
    '''
    if tensor.ndimension() < 2:
        raise ValueError("Only tensors with 2 or more dimensions are supported")

    rows = tensor.size(0)
    cols = tensor[0].numel()
    flattened = tensor.new_empty(rows, cols).normal_(0, 1)

    for i in range(0, rows, cols):
        # Compute the qr factorization
        q, r = torch.qr(flattened[i:i + cols].t())
        # Make Q uniform according to https://arxiv.org/pdf/math-ph/0609050.pdf
        q *= torch.diag(r, 0).sign()
        q.t_()

        with torch.no_grad():
            tensor[i:i + cols].view_as(q).copy_(q)

    with torch.no_grad():
        tensor.mul_(gain)
    return tensor


def linear(in_features, out_features, bias=True):
    '''
    Linear Module initialized properly
    '''
    m = nn.Linear(in_features, out_features, bias=bias)
    orthogonal_(m.weight)
    nn.init.zeros_(m.bias)
    return m


def play_and_train(args, policy, optim):
    total_loss = 0
    turns = 0
    scores = []

    while turns < args.bs:
        log_probs = []
        rewards = []

        game = Game(4)
        t = time_logging.start()
        while True:
            x = game.encode()
            t = time_logging.end("encode", t)
            x = torch.tensor(x, device=args.device, dtype=torch.float32)
            x = args.beta * policy(x)
            t = time_logging.end("policy", t)

            loss = [0]
            def sample(x, w=1):
                if torch.rand(()) < args.randmove:
                    m = torch.distributions.Categorical(logits=torch.zeros_like(x))
                else:
                    m = torch.distributions.Categorical(logits=x)
                i = m.sample().item()
                loss[0] += x.log_softmax(0)[i].mul(w)
                return i

            action = sample(x[:3])
            score = game.score

            if action == 0:
                position = sample(x[3:3+5])
                out = game.play(position)

            if action == 1:
                position = sample(x[3:3+5])
                out = game.discard(position)

            if action == 2:
                target = sample(x[3+5:3+5+5], 0.5)
                info = sample(x[3+5+5:3+5+5+10], 0.5)
                if info < 5:
                    out = game.clue(target, info)
                else:
                    out = game.clue(target, "rgbyp"[info-5])

            t = time_logging.end("decode", t)

            log_probs.append(loss[0])
            if out is not None:
                rewards.append(-1)
                break

            if game.gameover:
                if game.score == 25:
                    rewards.append(game.score - score)
                else:
                    rewards.append(-1)
                break

            rewards.append(game.score - score)

        if len(log_probs) >= 3:
            turns += len(log_probs)
            R = 0
            returns = []
            for r in rewards[::-1]:
                R = r + args.gamma * R
                returns.insert(0, R)
            returns = torch.tensor(returns, device=args.device, dtype=torch.float32)
            returns = (returns - returns.mean()) / (returns.std() + 1e-5)
            for log_prob, R in zip(log_probs, returns):
                total_loss += -(log_prob * R)

            scores.append(game.score)

    total_loss /= turns

    optim.zero_grad()
    total_loss.backward()
    optim.step()
    t = time_logging.end("backward & optim", t)

    return scores


def execute(args):
    torch.backends.cudnn.benchmark = True

    policy = nn.Sequential(
        linear(2270, args.n), Swish(),
        linear(args.n, args.n), Swish(),
        linear(args.n, args.n), Swish(),
        linear(args.n, args.n), Swish(),
        linear(args.n, 23)
    ).to(args.device)

    scores = [0]

    optim = torch.optim.Adam(policy.parameters(), lr=args.lr)

    if args.restore:
        with open(args.restore, 'rb') as f:
            torch.load(f)
            x = torch.load(f, map_location=args.device)
            scores = x['scores']
            policy.load_state_dict(x['state'])

    t = tqdm.tqdm()
    for i in itertools.count(1):
        new_scores = play_and_train(args, policy, optim)
        scores.extend(new_scores)

        if i % 1000 == 0:
            print()
            print(time_logging.text_statistics())
            yield {
                'args': args,
                'state': policy.state_dict(),
                'scores': scores,
            }

        avg_score = mean(scores[-args.n_avg:])
        t.update(len(new_scores))
        t.set_postfix_str("scores={} avg_score={:.2f}".format(scores[-5:], avg_score))

    t.close()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--lr", type=float, default=1e-5)
    parser.add_argument("--bs", type=int, default=10)
    parser.add_argument("--n", type=int, default=500)
    parser.add_argument("--n_avg", type=int, default=1000)
    parser.add_argument("--beta", type=float, default=0.01)
    parser.add_argument("--gamma", type=float, default=0.99)
    parser.add_argument("--randmove", type=float, default=0.4)
    parser.add_argument("--restore", type=str)

    parser.add_argument("--device", type=str, required=True)

    parser.add_argument("--pickle", type=str, required=True)
    args = parser.parse_args()

    new = True
    torch.save(args, args.pickle)
    try:
        for res in execute(args):
            with open(args.pickle, 'wb') as f:
                torch.save(args, f)
                torch.save(res, f)
                new = False
    except:
        if new:
            os.remove(args.pickle)
        raise

if __name__ == "__main__":
    main()
