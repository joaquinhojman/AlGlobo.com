#!/usr/bin/python3

# QUICK & DIRTY UTILITY 

import random

HEADER = 'id,hotel_cost,bank_cost,airline_cost\n'
FILE_PATH = 'transactions.csv'
NUM_TRANSACTIONS = 1000000
MIN_COST = 1000
MAX_COST = 2000

def main():
    with open(FILE_PATH, 'w') as f:
        f.write(HEADER)
        for id in range(NUM_TRANSACTIONS):
            nums = [random.randint(MIN_COST, MAX_COST) for _ in range(3)]
            f.write(f"{id},{nums[0]},{nums[1]},{nums[2]}\n")


if __name__ == '__main__':
    main()
