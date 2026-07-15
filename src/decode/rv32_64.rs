use crate::{
    decode::{
        formats::{fence_fm, fence_pred, fence_succ, funct3, funct5, funct6, funct7, imm_b, imm_i, imm_j, imm_u, imm12, opcode, rd, rs1, rs2, shamt5, shamt6, sign_extend, uimm},
        rv64fd::{self, decode_madd, decode_msub, decode_nmadd, decode_nmsub},
    },
    instruction::Instruction,
    opcode::{OP, OP_ATOMIC, OP_AUIPC, OP_BRANCH, OP_FP, OP_IMM, OP_IMM_W, OP_JAL, OP_JALR, OP_LOAD, OP_LOAD_FP, OP_LUI, OP_MISC_MEM, OP_STORE, OP_STORE_FP, OP_SYSTEM, OP_W, OPCODE_FMADD, OPCODE_FMSUB, OPCODE_FNMADD, OPCODE_FNMSUB},
};

pub fn decode_normal(raw: u32) -> Instruction {
    match opcode(raw) {
        OP_IMM => decode_op_imm(raw),
        OP_IMM_W => decode_op_imm_w(raw),
        OP => decode_op(raw),
        OP_W => decode_op_w(raw),
        OP_ATOMIC => decode_atomic(raw),
        OP_LOAD => decode_load(raw),
        OP_STORE => decode_store(raw),
        OP_BRANCH => decode_branch(raw),
        OP_JAL => decode_jal(raw),
        OP_JALR => decode_jalr(raw),
        OP_LUI => decode_lui(raw),
        OP_AUIPC => decode_auipc(raw),
        OP_SYSTEM => decode_system(raw),
        OP_MISC_MEM => decode_misc_mem(raw),
        // floating point
        OP_LOAD_FP => rv64fd::decode_load_fp(raw),
        OP_STORE_FP => rv64fd::decode_store_fp(raw),
        OP_FP => rv64fd::decode_op_fp(raw),
        OPCODE_FMADD => decode_madd(raw),
        OPCODE_FMSUB => decode_msub(raw),
        OPCODE_FNMSUB => decode_nmsub(raw),
        OPCODE_FNMADD => decode_nmadd(raw),

        _ => Instruction::Undefined { raw },
    }
}

fn decode_op_imm(raw: u32) -> Instruction {
    let rd = rd(raw);
    let rs1 = rs1(raw);
    let imm = imm_i(raw);
    let shamt = shamt6(raw);

    match funct3(raw) {
        0b000 => Instruction::Addi { rd, rs1, imm },
        0b010 => Instruction::Slti { rd, rs1, imm },
        0b011 => Instruction::Sltiu { rd, rs1, imm },
        0b100 => Instruction::Xori { rd, rs1, imm },
        0b110 => Instruction::Ori { rd, rs1, imm },
        0b111 => Instruction::Andi { rd, rs1, imm },
        0b001 => match funct6(raw) {
            0b000000 => Instruction::Slli { rd, rs1, shamt },
            _ => Instruction::Undefined { raw },
        },

        0b101 => match funct6(raw) {
            0b000000 => Instruction::Srli { rd, rs1, shamt },
            0b010000 => Instruction::Srai { rd, rs1, shamt },
            _ => Instruction::Undefined { raw },
        },
        _ => Instruction::Undefined { raw },
    }
}

fn decode_op_imm_w(raw: u32) -> Instruction {
    let rs1 = rs1(raw);
    let rd = rd(raw);
    let imm = imm_i(raw);
    let shamt = shamt5(raw);

    match funct3(raw) {
        0b000 => Instruction::Addiw { rd, rs1, imm },
        0b001 => Instruction::Slliw { rd, rs1, shamt },
        0b101 => match funct7(raw) {
            0b0000000 => Instruction::Srliw { rd, rs1, shamt },
            0b0100000 => Instruction::Sraiw { rd, rs1, shamt },
            _ => Instruction::Undefined { raw },
        },
        _ => Instruction::Undefined { raw },
    }
}

fn decode_op(raw: u32) -> Instruction {
    let rs1 = rs1(raw);
    let rs2 = rs2(raw);
    let rd = rd(raw);

    match (funct7(raw), funct3(raw)) {
        (0b0000000, 0b000) => Instruction::Add { rd, rs1, rs2 },
        (0b0100000, 0b000) => Instruction::Sub { rd, rs1, rs2 },
        (0b0000000, 0b001) => Instruction::Sll { rd, rs1, rs2 },
        (0b0000000, 0b010) => Instruction::Slt { rd, rs1, rs2 },
        (0b0000000, 0b011) => Instruction::Sltu { rd, rs1, rs2 },
        (0b0000000, 0b100) => Instruction::Xor { rd, rs1, rs2 },
        (0b0000000, 0b101) => Instruction::Srl { rd, rs1, rs2 },
        (0b0100000, 0b101) => Instruction::Sra { rd, rs1, rs2 },
        (0b0000000, 0b110) => Instruction::Or { rd, rs1, rs2 },
        (0b0000000, 0b111) => Instruction::And { rd, rs1, rs2 },
        // RV32M standard extension
        (0b0000001, 0b000) => Instruction::Mul { rd, rs1, rs2 },
        (0b0000001, 0b001) => Instruction::Mulh { rd, rs1, rs2 },
        (0b0000001, 0b010) => Instruction::Mulhsu { rd, rs1, rs2 },
        (0b0000001, 0b011) => Instruction::Mulhu { rd, rs1, rs2 },
        (0b0000001, 0b100) => Instruction::Div { rd, rs1, rs2 },
        (0b0000001, 0b101) => Instruction::Divu { rd, rs1, rs2 },
        (0b0000001, 0b110) => Instruction::Rem { rd, rs1, rs2 },
        (0b0000001, 0b111) => Instruction::Remu { rd, rs1, rs2 },

        _ => Instruction::Undefined { raw },
    }
}

fn decode_op_w(raw: u32) -> Instruction {
    let rs1 = rs1(raw);
    let rs2 = rs2(raw);
    let rd = rd(raw);

    match (funct7(raw), funct3(raw)) {
        (0b0000000, 0b000) => Instruction::Addw { rd, rs1, rs2 },
        (0b0100000, 0b000) => Instruction::Subw { rd, rs1, rs2 },
        (0b0000000, 0b001) => Instruction::Sllw { rd, rs1, rs2 },
        (0b0000000, 0b101) => Instruction::Srlw { rd, rs1, rs2 },
        (0b0100000, 0b101) => Instruction::Sraw { rd, rs1, rs2 },
        // RV64M word multiply/divide instruction
        (0b0000001, 0b000) => Instruction::Mulw { rd, rs1, rs2 },
        (0b0000001, 0b100) => Instruction::Divw { rd, rs1, rs2 },
        (0b0000001, 0b101) => Instruction::Divuw { rd, rs1, rs2 },
        (0b0000001, 0b110) => Instruction::Remw { rd, rs1, rs2 },
        (0b0000001, 0b111) => Instruction::Remuw { rd, rs1, rs2 },

        _ => Instruction::Undefined { raw },
    }
}

fn decode_atomic(raw: u32) -> Instruction {
    let rl = ((raw >> 25) & 1) != 0;
    let aq = ((raw >> 26) & 1) != 0;
    let rs1 = rs1(raw);
    let rs2 = rs2(raw);
    let rd = rd(raw);

    match (funct5(raw), funct3(raw)) {
        (0b00010, 0b010) => {
            if rs2.idx() != 0 {
                return Instruction::Undefined { raw };
            }
            Instruction::Lrw { rd, rs1, rl, aq }
        }
        (0b00011, 0b010) => Instruction::Scw { rd, rs1, rs2, rl, aq },
        (0b00001, 0b010) => Instruction::Amoswapw { rd, rs1, rs2, rl, aq },
        (0b00000, 0b010) => Instruction::Amoaddw { rd, rs1, rs2, rl, aq },
        (0b00100, 0b010) => Instruction::Amoxorw { rd, rs1, rs2, rl, aq },
        (0b01100, 0b010) => Instruction::Amoandw { rd, rs1, rs2, rl, aq },
        (0b01000, 0b010) => Instruction::Amoorw { rd, rs1, rs2, rl, aq },
        (0b10000, 0b010) => Instruction::Amominw { rd, rs1, rs2, rl, aq },
        (0b10100, 0b010) => Instruction::Amomaxw { rd, rs1, rs2, rl, aq },
        (0b11000, 0b010) => Instruction::Amominuw { rd, rs1, rs2, rl, aq },
        (0b11100, 0b010) => Instruction::Amomaxuw { rd, rs1, rs2, rl, aq },
        // RV64A atomic instructions
        (0b00010, 0b011) => {
            if rs2.idx() != 0 {
                return Instruction::Undefined { raw };
            }
            Instruction::Lrd { rd, rs1, rl, aq }
        }
        (0b00011, 0b011) => Instruction::Scd { rd, rs1, rs2, rl, aq },
        (0b00001, 0b011) => Instruction::Amoswapd { rd, rs1, rs2, rl, aq },
        (0b00000, 0b011) => Instruction::Amoaddd { rd, rs1, rs2, rl, aq },
        (0b00100, 0b011) => Instruction::Amoxord { rd, rs1, rs2, rl, aq },
        (0b01100, 0b011) => Instruction::Amoandd { rd, rs1, rs2, rl, aq },
        (0b01000, 0b011) => Instruction::Amoord { rd, rs1, rs2, rl, aq },
        (0b10000, 0b011) => Instruction::Amomind { rd, rs1, rs2, rl, aq },
        (0b10100, 0b011) => Instruction::Amomaxd { rd, rs1, rs2, rl, aq },
        (0b11000, 0b011) => Instruction::Amominud { rd, rs1, rs2, rl, aq },
        (0b11100, 0b011) => Instruction::Amomaxud { rd, rs1, rs2, rl, aq },

        _ => Instruction::Undefined { raw },
    }
}

fn decode_load(raw: u32) -> Instruction {
    let rd = rd(raw);
    let rs1 = rs1(raw);
    let imm = imm_i(raw);
    let funct3 = funct3(raw);

    match funct3 {
        0b000 => Instruction::Lb { rd, rs1, imm },
        0b001 => Instruction::Lh { rd, rs1, imm },
        0b010 => Instruction::Lw { rd, rs1, imm },
        0b100 => Instruction::Lbu { rd, rs1, imm },
        0b101 => Instruction::Lhu { rd, rs1, imm },
        // RV64I Load instructions
        0b110 => Instruction::Lwu { rd, rs1, imm },
        0b011 => Instruction::Ld { rd, rs1, imm },

        _ => Instruction::Undefined { raw },
    }
}

fn decode_store(raw: u32) -> Instruction {
    let rs1 = rs1(raw);
    let rs2 = rs2(raw);
    let imm11_5 = (raw >> 25) & 0b1111111;
    let imm4 = (raw >> 7) & 0b11111;
    let imm = sign_extend(((imm11_5 << 5) | imm4) as u32, 12);
    let funct3 = funct3(raw);

    match funct3 {
        0b000 => Instruction::Sb { rs2, rs1, imm },
        0b001 => Instruction::Sh { rs2, rs1, imm },
        0b010 => Instruction::Sw { rs2, rs1, imm },
        0b011 => Instruction::Sd { rs2, rs1, imm },
        _ => Instruction::Undefined { raw },
    }
}

fn decode_branch(raw: u32) -> Instruction {
    let imm = imm_b(raw);
    let rs1 = rs1(raw);
    let rs2 = rs2(raw);

    match funct3(raw) {
        0b000 => Instruction::Beq { rs1, rs2, imm },
        0b001 => Instruction::Bne { rs1, rs2, imm },
        0b100 => Instruction::Blt { rs1, rs2, imm },
        0b101 => Instruction::Bge { rs1, rs2, imm },
        0b110 => Instruction::Bltu { rs1, rs2, imm },
        0b111 => Instruction::Bgeu { rs1, rs2, imm },

        _ => Instruction::Undefined { raw },
    }
}

fn decode_jal(raw: u32) -> Instruction {
    let imm = imm_j(raw);
    let rd = rd(raw);

    Instruction::Jal { rd, imm }
}

fn decode_jalr(raw: u32) -> Instruction {
    let imm = imm_i(raw);
    let rd = rd(raw);
    let rs1 = rs1(raw);

    match funct3(raw) {
        0b000 => Instruction::Jalr { rd, rs1, imm },
        _ => Instruction::Undefined { raw },
    }
}

fn decode_lui(raw: u32) -> Instruction {
    let imm = imm_u(raw);
    let rd = rd(raw);

    Instruction::Lui { rd, imm }
}

fn decode_auipc(raw: u32) -> Instruction {
    let imm = imm_u(raw);
    let rd = rd(raw);

    Instruction::Auipc { rd, imm }
}

fn decode_system(raw: u32) -> Instruction {
    let funct3 = funct3(raw);
    let funct7 = funct7(raw);
    let csr = ((raw >> 20) & 0b111111111111) as u16;
    let rd = rd(raw);
    let rs1 = rs1(raw);
    let uimm = uimm(raw);
    let imm12 = imm12(raw);
    let rs2 = rs2(raw);

    match funct3 {
        0b000 => {
            // SFENCE.VMA
            if funct7 == 0b0001001 && rd.is_zero() {
                return Instruction::SfenceVma { rs1, rs2 };
            }

            match imm12 {
                0 => Instruction::Ecall,
                1 => Instruction::Ebreak,
                0x102 => Instruction::Sret,
                0x105 => Instruction::Wfi,
                0x302 => Instruction::Mret,
                _ => Instruction::Undefined { raw },
            }
        }
        0b001 => Instruction::Csrrw { rd, rs1, csr },
        0b010 => Instruction::Csrrs { rd, rs1, csr },
        0b011 => Instruction::Csrrc { rd, rs1, csr },
        0b101 => Instruction::Csrrwi { rd, uimm, csr },
        0b110 => Instruction::Csrrsi { rd, uimm, csr },
        0b111 => Instruction::Csrrci { rd, uimm, csr },

        _ => Instruction::Undefined { raw },
    }
}

fn decode_misc_mem(raw: u32) -> Instruction {
    let pred = fence_pred(raw);
    let succ = fence_succ(raw);
    let fm = fence_fm(raw);
    let imm12 = imm12(raw);
    let is_rs1_zero = rs1(raw).is_zero();
    let is_rd_zero = rd(raw).is_zero();

    match funct3(raw) {
        0b000 => Instruction::Fence { pred, succ, fm },

        0b001 => {
            if is_rd_zero && is_rs1_zero && imm12 == 0 {
                Instruction::FenceI
            } else {
                Instruction::Undefined { raw }
            }
        }

        _ => Instruction::Undefined { raw },
    }
}
